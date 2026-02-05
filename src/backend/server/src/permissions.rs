use actix_session::Session;
use actix_web::web;
use diesel::RunQueryDsl;
use diesel::prelude::*;
use log::warn;
use rand::Rng;
use std::str::FromStr;
use std::{fmt::Display, net::IpAddr, time::SystemTime};
use thiserror::Error;
use uuid::Uuid;

use crate::AppState;

// Surpassing 5 failed requests from one peer per 10 minutes will lead to a temporary block
pub const BASE_TIME_WINDOW: u64 = 10 * 60;
pub const BASE_REQUEST_LIMIT: usize = 5;

// Surpassing 15 failed requests from one peer per 30 minutes will lead to a permanent IP block
pub const LARGE_TIME_WINDOW: u64 = 30 * 60;
pub const LARGE_REQUEST_LIMIT: usize = 15;

// Surpassing 60 failed requests per 60 minutes will lead to the affected account being locked down for any IP
pub const ACCOUNT_TIME_WINDOW: u64 = 60 * 60;
pub const ACCOUNT_REQUEST_LIMIT: usize = 60;

// Duration in seconds that a session may be alive.
pub const SESSION_TIMEOUT: u64 = 60 * 60 * 12; // = 12h

#[derive(PartialEq, Debug, Clone)]
pub enum LoginAction {
    Limited,
    Rejected,
    Approved,
    Blocked,
}

impl Display for LoginAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Limited => f.write_str("LIMITED"),
            Self::Rejected => f.write_str("REJECTED"),
            Self::Blocked => f.write_str("BLOCKED"),
            Self::Approved => f.write_str("APPROVED"),
        }
    }
}

impl LoginAction {
    pub fn is_denying(&self) -> bool {
        matches!(
            self,
            LoginAction::Limited | LoginAction::Rejected | LoginAction::Blocked
        )
    }
}

#[derive(Debug, Clone)]
/// Describes a request to login in memory.
pub struct LoginRequest {
    pub time: SystemTime,
    pub action: LoginAction,
    pub ip: IpAddr,
    pub username: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Describes the session of a user that has logged in, in memory.
pub struct LoginSession {
    pub since: SystemTime,
    pub username: String,
    pub ip: IpAddr,
    pub token: String,
    pub id: String,
}

impl LoginSession {
    /// Checks if a session has not yet timed out.
    fn is_valid(&self) -> bool {
        SystemTime::now()
            .duration_since(self.since)
            .unwrap()
            .as_secs()
            <= SESSION_TIMEOUT
    }
}

#[derive(Debug, Error)]
/// Occurs when trying to locate a session by a malformed or wrong cookie.
pub enum SessionError {
    #[error("The backend was provided an invalid session id.")]
    WrongId,
    #[error("The backend was provided an invalid session token.")]
    WrongToken,
    #[error("The session state was not accessible.")]
    SessionStateInaccessible,
}

/// Given a target session (i.e. the session of the current request) and a reference to the
/// application state, the function will remove the target session from the state, not purging the
/// cookie session.
/// This effectively ends the login session on the server side.
///
/// The function will lock `state.sessions` until it has finished removing the session.
///
/// # Example
/// ```rust
/// let current = locate_session(&session, &state).expect("Missing a session!");
/// remove_session(current, &state);
/// ```
pub fn remove_session(target_session: LoginSession, state: &AppState) {
    state
        .sessions
        .lock()
        .unwrap()
        .retain(|e| *e != target_session);
}

/// Given the session cookie and a reference to the app state, this function will get a copy of the login
/// session corresponding to the session cookie.
/// This function will lock `state.sessions` until it has finished looking up the session.
///
/// # Example
/// ```rust
/// println!("Your username is {}", locate_session(&cookie, &state).unwrap().username)
/// ```
///
/// # Errors
/// The function will return a [`SessionError`] if the cookie is malformed or contains the wrong
/// token or id.
pub fn locate_session(
    cookie_session: &Session,
    state: &AppState,
) -> Result<LoginSession, SessionError> {
    if let Ok(parsed_for_id) = cookie_session.get::<String>("id")
        && let Some(id) = parsed_for_id
    {
        if let Ok(sessions) = state.sessions.lock() {
            let sessions_copy = sessions.clone();
            let mut with_matching_id = sessions_copy.iter().filter(|x| x.id == id);
            drop(sessions);
            if with_matching_id.clone().count() != 1 {
                return Err(SessionError::WrongId);
            }
            if let Ok(parsed_for_token) = cookie_session.get::<String>("login_token")
                && let Some(token) = parsed_for_token
                && let Some(with_matching_token) = with_matching_id.find(|x| x.token == token)
            {
                Ok(with_matching_token.clone())
            } else {
                Err(SessionError::WrongToken)
            }
        } else {
            warn!(
                "A session that may have existed could not be located, as the session state was locked."
            );
            Err(SessionError::SessionStateInaccessible)
        }
    } else {
        Err(SessionError::WrongId)
    }
}

/// Registers a new session on the server side and creates a session cookie.
/// The session uses a random token and is assigned an ID.
/// Having a session is the de-facto key to accessing restricted routes.
///
/// # Example
/// ```rust
/// register_session(&cookie, &state, "admin", IpAddr::from("127.0.0.1"));
/// ```
pub fn register_session(session: Session, state: &AppState, username: String, ip: IpAddr) {
    let random_token = generate_random_token();
    let id = Uuid::new_v4().to_string();

    state.sessions.lock().unwrap().push(LoginSession {
        since: SystemTime::now(),
        username,
        ip,
        token: random_token.clone(),
        id: id.clone(),
    });
    let _ = session.insert("login_token", random_token);
    let _ = session.insert("id", id);
}

/// Checks if a user is admin.
///
/// The function requires two arguments:
/// * `session` - The current session from the handler
/// * `state` - The current server state in form of the AppState struct.
///
/// The function will then compare the states login token to the token provided by the session.
/// In case no token is provided by the sessions, false is returned as default.
pub fn is_privileged(session: &Session, state: web::Data<AppState>) -> bool {
    if let Ok(current_session) = locate_session(session, &state) {
        if current_session.is_valid() {
            true
        } else {
            remove_session(current_session, &state);
            session.purge();
            false
        }
    } else {
        false
    }
}

/// Adds an IP address to the list of blocked peers both in the app state and the database.
pub fn block_ip(state: &AppState, to_be_blocked_ip: IpAddr) {
    use utils::models::BlockedIp;
    use utils::schema::BlockedIPs::dsl::*;

    let exec = diesel::insert_into(BlockedIPs)
        .values(BlockedIp {
            ip: to_be_blocked_ip.to_string(),
            since: utils::time::current_timestamp_unix() as i64,
        })
        .execute(&mut state.db_pool.lock().unwrap().get().unwrap());

    if exec.is_err() {
        log::error!("Failed to store IP {to_be_blocked_ip} as blocked in database.");
    }

    state.blocked_ips.lock().unwrap().push(to_be_blocked_ip);
}

/// Checks if an IP address is in the app states list of blocked ips.
pub fn is_blocked_ip(state: &AppState, ip: IpAddr) -> bool {
    state.blocked_ips.lock().unwrap().contains(&ip)
}

/// Given a reference of the app state, this function will insert all blocked IPs from the database into
/// the app state.
pub fn load_blocked_ips(state: &AppState) {
    use utils::models::BlockedIp;
    use utils::schema::BlockedIPs::dsl::*;

    let blocked_ips: Vec<BlockedIp> = BlockedIPs
        .select(BlockedIp::as_select())
        .get_results(&mut state.db_pool.lock().unwrap().get().unwrap())
        .unwrap();

    state.blocked_ips.lock().unwrap().append(
        &mut blocked_ips
            .iter()
            .map(|x| IpAddr::from_str(&x.ip).unwrap())
            .collect::<Vec<IpAddr>>(),
    )
}

/// Generates a new random string that can be used as a session token.
pub fn generate_random_token() -> String {
    let mut rng = rand::rngs::OsRng;
    let token: [u8; 32] = rng.r#gen();
    hex::encode(token)
}
