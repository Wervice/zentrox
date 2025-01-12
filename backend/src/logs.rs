use crate::sudo::{SudoExecutionOutput, SwitchedUserCommand};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
/// Every entry in journalctl is parsed into this struct.
/// The struct uses Option<T> instead of T, because it is not guaranteed that every field is
/// actually available.
struct JournalEntry {
    #[serde(rename = "__REALTIME_TIMESTAMP")]
    realtime_timestamp: Option<String>,

    #[serde(rename = "MESSAGE")]
    message: Option<String>,

    #[serde(rename = "PRIORITY")]
    priority: Option<String>,

    #[serde(rename = "_UID")]
    uid: Option<String>,

    #[serde(rename = "USER")]
    user: Option<String>,

    #[serde(rename = "USERNAME")]
    username: Option<String>,

    #[serde(rename = "SYSLOG_IDENTIFIER")]
    application: Option<String>,
}

/// Parse journalctl into a vector of messages
/// * `sudo_password` Password used to invoce journalctl
/// * `since` A UNIX timestamp where the log starts
/// * `until` A UNIX timestamp where the log ends
pub fn log_messages(
    sudo_password: String,
    since: u64,
    until: u64,
) -> Result<Vec<(String, String, String, String, String)>, String> {
    let jctl = SwitchedUserCommand::new(sudo_password, "journalctl".to_string())
        .arg("-o".to_string())
        .arg("json".to_string())
        .arg("--since".to_string())
        .arg("@".to_string() + &since.to_string())
        .arg("--until".to_string())
        .arg("@".to_string() + &until.to_string())
        .output();

    let v = match jctl {
        SudoExecutionOutput::Success(ou) => {
            let o = ou.stdout;
    let mut vect = Vec::new();
    for l in o.lines() {
        let entry: JournalEntry = serde_json::from_str(l).unwrap_or(JournalEntry {
            realtime_timestamp: Some(String::new()),
            application: Some(String::new()),
            message: Some(String::from(
                "This appears to be an error in the logs' data",
            )),
            priority: Some(String::new()),
            uid: Some(String::new()),
            user: Some(String::new()),
            username: Some(String::new()),
        });
        vect.push((
            entry.realtime_timestamp.unwrap_or(String::from("")),
            {
                // Figure out who invoced the message.
                // Journalctl may provide this information in one of these three fields:
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
    vect
        }
        _ => return Err("Failed to invoke journalctl".to_string()),
    };

    Ok(v)
    // Pattern: [[TIMESTAMP, USERNAME, APPLICATION_THAT_INVOKED_THE_MESSAGE, MESSAGE, PRIORITY AS
    //DIGIT],...]
}
