//! Bindings for `systemd` through the `systemctl` command.

use serde::Deserialize;
use std::{collections::HashMap, error::Error, path::PathBuf, process::Command};
use sysinfo::{self, Pid};

use crate::sudo::SudoCommand;

// TODO Replace that with D-Bus based communication with org.freedesktop.systemd1

#[derive(Default, Debug)]
pub enum Restart {
    OnSuccess,
    OnFailure,
    OnWatchdog,
    OnAbort,
    Always,
    #[default]
    No,
}

impl From<&str> for Restart {
    fn from(value: &str) -> Self {
        match value {
            "on-success" => Restart::OnSuccess,
            "on-failure" => Restart::OnFailure,
            "on-watchdog" => Restart::OnWatchdog,
            "on-abort" => Restart::OnAbort,
            "always" => Restart::Always,
            "no" => Restart::No,
            _ => panic!("Unknown restart setting."),
        }
    }
}

#[derive(Default, Debug)]
pub enum Activity {
    Active,
    Reloading,
    #[default]
    Inactive,
    Failed,
    Activating,
    Deactivating,
    Maintenance,
    Refreshing,
}

impl From<&str> for Activity {
    fn from(value: &str) -> Self {
        match value {
            "active" => Self::Active,
            "reloading" => Self::Reloading,
            "inactive" => Self::Inactive,
            "failed" => Self::Failed,
            "activating" => Self::Activating,
            "deactivating" => Self::Deactivating,
            "maintenance" => Self::Maintenance,
            "refreshing" => Self::Refreshing,
            _ => panic!("Unknown activity setting."),
        }
    }
}

#[derive(Default, Debug)]
pub enum UnitFileState {
    Enabled,
    EnabledRuntime,
    Linked,
    LinkedRuntime,
    Alias,
    Masked,
    MaskedRuntime,
    Static,
    #[default]
    Disabled,
    Indirect,
    Generated,
    Transient,
    Bad,
}

impl From<&str> for UnitFileState {
    fn from(value: &str) -> Self {
        match value {
            "enabled" => Self::Enabled,
            "enabled-runtime" => Self::EnabledRuntime,
            "linked" => Self::Linked,
            "linked-runtime" => Self::LinkedRuntime,
            "alias" => Self::Alias,
            "masked" => Self::Masked,
            "masked-runtime" => Self::MaskedRuntime,
            "static" => Self::Static,
            "disabled" => Self::Disabled,
            "indirect" => Self::Indirect,
            "generated" => Self::Generated,
            "transient" => Self::Transient,
            "bad" => Self::Bad,
            _ => panic!("Unknown unit file state."),
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub enum LoadState {
    #[default]
    Stub,
    Loaded,
    NotFound,
    BadSetting,
    Error,
    Merged,
    Masked,
}

impl From<&str> for LoadState {
    fn from(value: &str) -> Self {
        match value {
            "stub" => Self::Stub,
            "loaded" => Self::Loaded,
            "not-found" => Self::NotFound,
            "bad-setting" => Self::BadSetting,
            "error" => Self::Error,
            "merged" => Self::Merged,
            "masked" => Self::Masked,
            _ => panic!("Unknown loading state."),
        }
    }
}

#[derive(Default, Debug)]
/// Provides the most important information for a `systemd` daemon.
/// The data is retrieved using `systemctl`.
pub struct Daemon {
    names: Vec<String>,
    id: String,
    active: Activity,
    enabled: UnitFileState,
    restart: Restart,
    loaded: LoadState,
    description: Option<String>,
    pid: Option<u32>,
}

/// Verifies if a given ID can safely be passed into `systemctl`.
fn sanitize_id(id: String) -> Option<String> {
    if id.len() > 255 {
        return None;
    }

    if id.chars().filter(|c| c == &'.').count() > 1 {
        return None;
    }

    if !id.ends_with(".service") {
        return None;
    }

    if id.chars().any(|c| !c.is_ascii_alphanumeric() && c != '.') {
        return None;
    }

    if !PathBuf::from("/usr/lib/systemd/system").join(&id).exists() {
        return None;
    }

    Some(id)
}

impl Daemon {
    /// Attempts to create a [`Daemon`] struct by looking up a specified unit file `id` (i.e. `docker.service`) using `systemctl`.
    pub fn try_from_id(id: String) -> Result<Daemon, Box<dyn Error>> {
        status(id)
    }

    /// Checks if a daemon is active.
    pub fn is_active(&self) -> bool {
        matches!(self.active, Activity::Active)
    }

    /// Provides the current activity status of a daemon.
    pub fn activity(&self) -> Result<Activity, Box<dyn Error>> {
        Ok(status(self.id.clone())?.active)
    }

    /// Provides the full ID of a daemon.
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Provides all names of a daemon.
    pub fn names(&self) -> Vec<String> {
        self.names.clone()
    }

    /// Provides the name of a daemon.
    pub fn name(&self) -> String {
        self.names.first().unwrap().to_string()
    }

    /// Provides the UnitFileState of a daemon, effectively determining if the daemon is enabled or
    /// disabled, while [`UnitFileState`] also provides less common variants.
    pub fn enabled_setting(&self) -> Result<UnitFileState, Box<dyn Error>> {
        Ok(status(self.id.clone())?.enabled)
    }

    /// Provides the configuration by which is determined under which conditions to restart a
    /// daemon.
    pub fn restart_setting(&self) -> Result<Restart, Box<dyn Error>> {
        Ok(status(self.id.clone())?.restart)
    }

    /// Provides the current state of loading a daemon.
    pub fn load_state(&self) -> Result<LoadState, Box<dyn Error>> {
        Ok(status(self.id.clone())?.loaded)
    }

    /// If present, provides a description of a daemon.
    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    /// Runs a command of the scheme `systemctl ARGUMENT ID` with sudo.
    /// The argument (`ARGUMENT`) must be passed into the function as a string.
    fn single_argument(&self, argument: &str, sudo_password: String) -> Result<(), Box<dyn Error>> {
        if argument.chars().any(|c| !c.is_ascii_alphabetic()) {
            return Err("An argument may only be of alphabetic characters.".into());
        }

        let mut c = SudoCommand::new(sudo_password, "systemctl");
        c.args(vec![
            argument,
            sanitize_id(self.id.to_string())
                .ok_or("No such service service daemon exists.")?
                .as_str(),
        ]);

        let o = c.output()?;
        let status_code = o
            .status
            .ok_or("Failed to retrieve status code from sudo.")?;

        if status_code != 0 {
            return Err("The systemctl command failed.".into());
        }

        Ok(())
    }

    /// Restarts a daemon using sudo.
    pub fn restart(&self, sudo_password: String) -> Result<(), Box<dyn Error>> {
        self.single_argument("restart", sudo_password)
    }

    /// Stops a daemon using sudo.
    pub fn stop(&self, sudo_password: String) -> Result<(), Box<dyn Error>> {
        self.single_argument("stop", sudo_password)
    }

    /// Starts a daemon using sudo.
    pub fn start(&self, sudo_password: String) -> Result<(), Box<dyn Error>> {
        self.single_argument("start", sudo_password)
    }

    /// Enables a daemon using sudo.
    pub fn enable(&self, sudo_password: String) -> Result<(), Box<dyn Error>> {
        self.single_argument("enable", sudo_password)
    }

    /// Disables a daemon using sudo.
    pub fn disable(&self, sudo_password: String) -> Result<(), Box<dyn Error>> {
        self.single_argument("disable", sudo_password)
    }

    /// If present, returns with PID for a daemon.
    pub fn pid(&self) -> Option<sysinfo::Pid> {
        self.pid.map(Pid::from_u32)
    }
}

/// Provides common details for an active service by its unit specifier (i.e. docker.service).
fn status(id: String) -> Result<Daemon, Box<dyn Error>> {
    let mut c = Command::new("systemctl");
    c.args([
        "show",
        "--no-pager",
        sanitize_id(id)
            .ok_or("No such service daemon exists.")?
            .as_str(),
    ]);

    let output = c.output()?;

    if !output.status.success() {
        return Err("Failed to execute systemctl command.".into());
    }

    let stdout = String::from_utf8(output.stdout)?;
    let kv_pairs: HashMap<&str, &str> = stdout
        .lines()
        .filter_map(|e| {
            let mut segments = e.split('=');
            if segments.clone().count() != 2 {
                return None;
            }
            Some((segments.next().unwrap(), segments.next_back().unwrap()))
        })
        .collect();

    let loaded = LoadState::from(*kv_pairs.get("LoadState").ok_or("No LoadState value.")?);

    if loaded == LoadState::NotFound {
        return Err("No such daemon exists.".into());
    }

    let id = kv_pairs.get("Id").ok_or("No Id value.")?.to_string();
    let names: Vec<String> = kv_pairs
        .get("Names")
        .ok_or("No Names value.")?
        .split(' ')
        .map(String::from)
        .collect();
    let active = Activity::from(*kv_pairs.get("ActiveState").ok_or("No ActiveState value.")?);
    let enabled = UnitFileState::from(
        *kv_pairs
            .get("UnitFileState")
            .ok_or("No UnitFileState value.")?,
    );

    let mut pid: Option<u32> = None;

    if let Some(value) = kv_pairs.get("MainPID")
        && pid.is_none()
    {
        pid = value.parse::<u32>().ok()
    }

    if let Some(value) = kv_pairs.get("ExecMainPID")
        && pid.is_none()
    {
        pid = value.parse::<u32>().ok()
    }

    let restart = Restart::from(*kv_pairs.get("Restart").ok_or("No Restart value.")?);

    let description = kv_pairs.get("Description").map(|e| e.to_string());

    Ok(Daemon {
        names,
        id,
        active,
        enabled,
        restart,
        loaded,
        description,
        pid,
    })
}

#[derive(Deserialize)]
struct ShortService {
    unit: String,
    // More fields are present in the actual output, but those are not necessary.
}

/// Provides a list of all active services by their names.
pub fn name_list() -> Result<Vec<String>, Box<dyn Error>> {
    let mut c = Command::new("systemctl");
    c.arg("--output=json");
    let o = c.output()?;
    let stdout = String::from_utf8(o.stdout)?;

    let services = serde_json::from_str::<Vec<ShortService>>(&stdout)?;
    Ok(services
        .iter()
        .filter_map(|e| {
            if e.unit.ends_with(".service") {
                Some(e.unit.split('.').next().unwrap().to_string())
            } else {
                None
            }
        })
        .collect())
}

// TODO Surely, there are more fields of daemons that have not been covered here
