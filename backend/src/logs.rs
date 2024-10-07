use crate::sudo::SwitchedUserCommand;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct JournalEntry {
    #[serde(rename = "__REALTIME_TIMESTAMP")]
    realtime_timestamp: Option<String>,

    #[serde(rename = "MESSAGE")]
    message: Option<String>,

    #[serde(rename = "_HOSTNAME")]
    hostname: Option<String>,

    #[serde(rename = "PRIORITY")]
    priority: Option<String>,

    #[serde(rename = "_UID")]
    uid: Option<String>,
    
    #[serde(rename = "USER")]
    user: Option<String>,

    #[serde(rename = "USERNAME")]
    username: Option<String>,

    #[serde(rename = "_MACHINE_ID")]
    machine_id: Option<String>,

    #[serde(rename = "_PID")]
    pid: Option<String>,

    #[serde(rename = "_GID")]
    gid: Option<String>,

    #[serde(rename = "SYSLOG_IDENTIFIER")]
    application: Option<String>,
}

pub fn log_messages(
    sudo_password: String, duration: u64
) -> Result<Vec<(String, String, String, String, String)>, String> {
    let jctl = SwitchedUserCommand::new(sudo_password, "journalctl".to_string())
        .arg("-o".to_string())
        .arg("json".to_string())
        .arg("--since".to_string())
        .arg("@".to_string() + &duration.to_string())
        .output();
    let o = jctl.unwrap().stdout;
    let mut vect = Vec::new();
    for l in o.lines() {
        let entry: JournalEntry = serde_json::from_str(l).unwrap();
        vect.push((
            entry.realtime_timestamp.unwrap_or(String::from("")),
            {
                if entry.user.is_some() {
                    entry.user.unwrap()
                } else if entry.username.is_some() {
                     entry.username.unwrap()
                } else if entry.uid.is_some() {
                     entry.uid.unwrap()
                } else {
                    String::from("Unknown Username").to_string()
                }
            },
            entry.application.unwrap_or(String::from("")),
            entry.message.unwrap_or(String::from("")),
            entry.priority.unwrap_or(String::from("")),
        ))
    }

    Ok(vect)
}
