//! Authorize the current process to perform certain polkit actions by registering a temporary
//! authentication agent (Polkit term, not to be confused with [`AuthenticationPortal`]) and providing the given password to Polkit.
//!
//! For this code to compile, the following packages must be present on the machine:
//!     - `libpolkit-gobject-1-dev` (LGPL-2.0+ and Expat)
//!     - `glib-2.0` (GPL-2.0-or-later)
//!     - `libdbus-1-dev` (GPL-2+ or AFL-2.1, and Expat and Tcl-BSDish)
//!     - `libpolkit-agent-1-dev` (LGPL-2.0+ and Expat)

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dbus::Message;
use dbus::arg::{RefArg, Variant};
use dbus::blocking::BlockingSender;
use glib::object::{Cast, ObjectExt};
use glib::subclass::prelude::*;
use glib::{MainLoop, SendWeakRef};
use polkit_agent_rs::gio::prelude::CancellableExtManual;
use polkit_agent_rs::subclass::ListenerImpl;
use polkit_agent_rs::traits::ListenerExt;
use polkit_agent_rs::{RegisterFlags, Session, gio};
use thiserror::Error;
use uuid::Uuid;

// TODO Propagate wrong password error somehow
// TODO D-Bus requests do not arrive after one wrong password (what does actually segfault when the
// password is wrong? Is it maybe the authorization agent??)
// TODO Use this to replace `sudo.rs` via `pkexec`

#[derive(Default)]
pub struct ZListenerPriv {
    active_attempts: RefCell<Arc<Mutex<Vec<Attempt>>>>,
    session: Arc<Mutex<Option<Session>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ZListenerPriv {
    const NAME: &'static str = "ZListener";
    type Type = ZListener;
    type ParentType = polkit_agent_rs::Listener;
}

impl ObjectImpl for ZListenerPriv {}

glib::wrapper! {
    pub struct ZListener(ObjectSubclass<ZListenerPriv>)
        @extends polkit_agent_rs::Listener;
}

impl ListenerImpl for ZListenerPriv {
    type Message = bool;

    fn initiate_authentication(
        &self,
        action_id: &str,
        _message: &str,
        _icon_name: &str,
        _details: &polkit_agent_rs::polkit::Details,
        cookie: &str,
        identities: Vec<polkit_agent_rs::polkit::Identity>,
        cancellable: gio::Cancellable,
        task: gio::Task<Self::Message>,
    ) {
        log::debug!("Initiating authentication for {action_id}.");
        let identity = identities.first().unwrap();
        log::debug!(
            "Authentication agent: Using identity with username \"{}\"",
            identity
                .clone()
                .downcast_ref::<polkit_agent_rs::polkit::UnixUser>()
                .and_then(|x| x.name())
                .unwrap_or("Error: No username".into())
        );

        let sess = Session::new(identity, cookie);
        *self.session.lock().unwrap() = Some(sess.clone());
        let task_weak_ref = SendWeakRef::from(task.downgrade());
        if let Some(lookup) = self
            .active_attempts
            .borrow()
            .lock()
            .unwrap()
            .iter()
            .find(|x| x.action_id == action_id)
        {
            let pw = lookup.password.clone();
            let uuid = lookup.uuid;
            let active_attempts_clone = self.active_attempts.clone();
            sess.connect_request(move |sess, _, _| {
                log::info!("Authentication agent: Received request and entering password.");
                active_attempts_clone
                    .borrow()
                    .lock()
                    .unwrap()
                    .retain(|x| x.uuid != uuid);
                sess.response(pw.as_str());
            });
            sess.connect_completed(move |_, gained_auth| {
                log::info!("Authentication agent: Completed with gained auth {gained_auth}");
                unsafe {
                    task_weak_ref
                        .upgrade()
                        .unwrap()
                        .return_result(Ok(gained_auth))
                }
            });
            sess.connect_show_info(|_, information| {
                // This is a warn, because it is not expected for the agent to have to display information.
                log::warn!("Authentication agent: Requested to show information: {information}");
            });
            sess.connect_show_error(|_, error| {
                log::error!("Authentication agent: Requested to show error: {error}");
            });
            let _ = cancellable.connect_cancelled(move |_| {
                log::error!("Authentication agent: Cancelled");
            });
            sess.initiate();
        } else {
            log::debug!("Authentication agent: Looked for {action_id}.");
            log::error!(
                "Authentication agent: Failed to find a suiting combination of password and action id. Aborting setting up an authentication agent. Request will timeout or block."
            )
        }
    }

    fn initiate_authentication_finish(
        &self,
        gio_result: Result<gio::Task<Self::Message>, gio::glib::Error>,
    ) -> bool {
        match gio_result {
            Ok(_task) => {
                true
            }
            Err(err) => {
                log::error!("Authentication agent: Received error: {err}");
                false
            }
        }
    }
}

#[derive(Clone, Debug)]
/// An (authentication) Attempt is used to describe a combination of an action_id and a password.
/// Attempts are stored inside the [`ZListenerPriv`] awaiting a call from Polkit to provide a password
/// for a specific action_id.
/// Attempts will be destroyed by the [`ZListenerPriv`] after 10 seconds or as soon as they have
/// been used to authenticate to Polkit.
struct Attempt {
    /// The ID of an action known to Polkit
    action_id: String,
    /// A password that can be used to authenticate
    password: String,
    uuid: Uuid,
    /// The at which the `Attempt` was created.
    creation_time: Instant,
}

#[derive(Clone, Debug)]
/// The `AuthenticationPortal` is responsible for creating a [`ZListener`] configured to the current process.
/// It holds the current password attempts for authentication.
pub struct AuthenticationPortal {
    mainloop: Arc<Mutex<MainLoop>>,
    attempts: Arc<Mutex<Vec<Attempt>>>,
}

/// Brief description of an action that is configured for Polkit.
#[derive(Debug, Clone)]
pub struct ActionDescription {
    pub action_id: String,
    pub description: String,
    pub message: String,
    pub vendor_name: String,
    pub vendor_url: String,
    pub icon_name: String,
    pub implicit_any: u32,
    pub implicit_active: u32,
    pub implicit_inactive: u32,
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("No such action_id is known to PolicyKit1.")]
    NoSuchActionId(String),
    #[error(
        "All rx channels have already closed, most likely because the listener shut down already."
    )]
    ListenerShutDown,
    #[error("The program is already authorized and a request is not needed.")]
    AlreadyAuthorised,
    #[error("Failed to check if the program is already authorized.")]
    CheckFailed,
}

impl Default for AuthenticationPortal {
    fn default() -> Self {
        Self {
            mainloop: Arc::new(Mutex::new(MainLoop::new(None, false))),
            attempts: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl AuthenticationPortal {
    /// Starts the authentication portal, creating a [`ZListener`].
    /// This function will not block.
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let ml = self.mainloop.clone();
        let attempts_clone_listener = self.attempts.clone();
        let attempts_clone_auto_clean = self.attempts.clone();

        std::thread::spawn(move || {
            loop {
                // Remove any attempt after 10 seconds, in order to clear sensitive
                // information out of the memory as soon as reasonable.
                attempts_clone_auto_clean
                    .lock()
                    .unwrap()
                    .retain(|att| att.creation_time.elapsed().as_secs() <= 10);
                std::thread::sleep(Duration::from_millis(100));
            }
        });

        std::thread::spawn(move || {
            let pid = std::process::id();
            let uid = crate::users::NativeUser::from_username(
                whoami::username().expect("No user could be detected."),
            );
            let creation_time = fs::read_to_string("/proc/self/stat")
                .expect("Kernel did not provide self process.")
                .split_whitespace()
                .nth(21)
                .expect("Kernel did not provide start time.")
                .parse::<u64>()
                .unwrap();

            log::debug!("Authentication agent: Creating new subject for pid={pid}");

            // The subject for which to create the listener.
            let subject = polkit_agent_rs::polkit::UnixProcess::new_for_owner(
                pid as i32,
                creation_time,
                uid.unwrap().user_id as i32,
            );

            let l: ZListener = gio::glib::Object::new(); // Create a glib object for the subclass
            // ZListener.
            log::debug!("Authentication agent: Setting up new listener.");

            // Registers the listener l for the subject.
            // Dropping the result of the call causes the listener to be unregistered.
            let r = l.register(
                RegisterFlags::NONE,
                &subject,
                "/org/freedesktop/PolicyKit1/AuthenticationAgent",
                None::<&gio::Cancellable>,
            );

            *l.imp().active_attempts.borrow_mut() = attempts_clone_listener;

            if r.is_err() {
                log::error!("Authentication agent: Failed to register listener.");
                return;
            }

            ml.lock().unwrap().run();
        });

        Ok(())
    }

    /// Provides a password to the [`AuthenticationPortal`] and thus to the listener. As soon as
    /// PolicyKit requests credentials from the authentication agent, the listener will be able to
    /// full fill this request.
    ///
    /// * `action_id` - The action_id representing an action known to PolicyKit
    /// * `password` - The password of the administrator account
    ///
    /// # Errors
    /// The action will fail if the provided `action_id` is not known to PolicyKit or the program
    /// already has adequate authorization.
    pub fn provide_password(
        &mut self,
        action_id: String,
        password: String,
    ) -> Result<(), RequestError> {
        match Self::check_is_authorized(&action_id) {
            Ok(b) => {
                if b {
                    return Err(RequestError::AlreadyAuthorised);
                }
            }
            Err(_) => return Err(RequestError::CheckFailed),
        }
        let actions = AuthenticationPortal::enumerate_actions().unwrap();
        if !actions.iter().any(|x| x.action_id == action_id) {
            log::warn!(
                "Authentication agent: An authentication attempt for an unknown action_id will be ignored."
            );
            return Err(RequestError::NoSuchActionId(action_id));
        }

        let uuid = Uuid::new_v4();

        self.attempts.lock().unwrap().push(Attempt {
            action_id,
            password,
            uuid,
            creation_time: Instant::now(),
        });

        Ok(())
    }

    /// Provides a list of all actions known to Polkit.
    pub fn enumerate_actions() -> Result<Vec<ActionDescription>, Box<dyn std::error::Error>> {
        type RawAction = (
            String,
            String,
            String,
            String,
            String,
            String,
            u32,
            u32,
            u32,
            HashMap<String, String>,
        );

        let conn = dbus::blocking::Connection::new_system()?;
        let m = Message::new_method_call(
            "org.freedesktop.PolicyKit1",
            "/org/freedesktop/PolicyKit1/Authority",
            "org.freedesktop.PolicyKit1.Authority",
            "EnumerateActions",
        )?
        .append1("rw");
        let r = conn.send_with_reply_and_block(m, Duration::from_secs(10))?;

        let args: Vec<RawAction> = r.get1().expect("PolicyKit did not supply enough outputs.");

        Ok(args
            .iter()
            .map(|x| ActionDescription {
                action_id: x.0.clone(),
                description: x.1.clone(),
                vendor_url: x.2.clone(),
                message: x.3.clone(),
                vendor_name: x.4.clone(),
                icon_name: x.5.clone(),
                implicit_any: x.6,
                implicit_inactive: x.7,
                implicit_active: x.8,
            })
            .collect())
    }

    pub fn check_is_authorized(action_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let actions = Self::enumerate_actions()?;
        if !actions.iter().any(|x| x.action_id == action_id) {
            return Err("No such action exists.".into());
        }

        let creation_time = fs::read_to_string("/proc/self/stat")
            .expect("Kernel did not provide self process.")
            .split_whitespace()
            .nth(21)
            .expect("Kernel did not provide start time.")
            .parse::<u64>()
            .unwrap();

        let mut subject_details: HashMap<String, Variant<Box<dyn RefArg>>> = HashMap::new();

        subject_details.insert("pid".to_string(), Variant(Box::new(std::process::id())));

        subject_details.insert("start-time".to_string(), Variant(Box::new(creation_time)));

        let subject = ("unix-process", subject_details);

        let details: HashMap<String, String> = HashMap::new();

        let flags = 0_u32;

        let cancellation_id = String::new();

        let conn = dbus::blocking::Connection::new_system()?;
        let m = Message::new_method_call(
            "org.freedesktop.PolicyKit1",
            "/org/freedesktop/PolicyKit1/Authority",
            "org.freedesktop.PolicyKit1.Authority",
            "CheckAuthorization",
        )?
        .append1(subject)
        .append1(action_id.to_string())
        .append1(details)
        .append1(flags)
        .append1(cancellation_id);
        let r = conn.send_with_reply_and_block(m, Duration::from_secs(10))?;

        let args: (bool, bool, Vec<(String, String)>) =
            r.get1().expect("Did not receive a file descriptor.");

        Ok(args.0)
    }
}
