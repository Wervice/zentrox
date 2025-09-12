use std::fmt::Display;

use crate::sudo::{SudoExecutionOutput, SwitchedUserCommand};
use crate::users::NativeUser;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
/// Journalctl may store log messages as arrays (JSON; Rust "Vec<T>") of integers that correspond
/// to bytes. To decode this, a phantom value is used.
enum PhantomStringOrBytes {
    String(String),
    Bytes(Vec<u8>),
}

impl Display for PhantomStringOrBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::Bytes(b) => {
                String::from_utf8(b.to_vec()).expect("Invalid bytes in journalctl log.")
            }
            Self::String(s) => s.to_string(),
        };
        f.write_str(&string)
    }
}

#[derive(Debug, Deserialize)]
/// Every entry in journalctl is parsed into this struct.
/// The struct uses Option<T> instead of T, because it is not guaranteed that every field is
/// actually available.
struct JournalEntryDeserializationSchema {
    #[serde(rename(deserialize = "__REALTIME_TIMESTAMP"))]
    pub realtime_timestamp: String,

    #[serde(rename(deserialize = "MESSAGE"))]
    pub message: Option<PhantomStringOrBytes>,

    #[serde(rename(deserialize = "PRIORITY"))]
    pub priority: Option<String>,

    #[serde(rename(deserialize = "_UID"))]
    pub uid: Option<String>,

    #[serde(rename(deserialize = "USER"))]
    pub user: Option<String>,

    #[serde(rename(deserialize = "USERNAME"))]
    pub username: Option<String>,

    #[serde(rename(deserialize = "SYSLOG_IDENTIFIER"))]
    pub application: Option<String>,
}

#[derive(Serialize, Debug, ToSchema, Clone)]
pub struct JournalEntry {
    pub timestamp: u128,
    pub message: Option<String>,
    pub priority: Option<String>,
    pub user: Option<NativeUser>,
    pub application: Option<String>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct QuickJournalEntry {
    pub timestamp: u128,
    pub message: Option<String>,
    pub priority: Option<String>,
    pub user: Option<u32>,
    pub application: Option<String>,
}

impl JournalEntry {
    pub fn as_quick_journal_entry(self) -> QuickJournalEntry {
        return QuickJournalEntry {
            timestamp: self.timestamp,
            message: self.message,
            priority: self.priority,
            user: {
                match self.user {
                    Some(user) => Some(user.user_id),
                    None => None,
                }
            },
            application: self.application,
        };
    }
}

#[derive(Debug)]
pub enum LogMessageError {
    InvocationError, // The program failed to start journalctl
    BadJournalEntryStructure(),
}

/// Parse journalctl into a vector of messages
/// * `sudo_password` Password used to invoce journalctl
/// * `since` A UNIX timestamp where the log starts
/// * `until` A UNIX timestamp where the log ends
pub fn log_messages(
    sudo_password: String,
    since: u64,
    until: u64,
) -> Result<Vec<JournalEntry>, LogMessageError> {
    let journalctl_command = SwitchedUserCommand::new(sudo_password, "journalctl".to_string())
        .arg("-o".to_string())
        .arg("json".to_string())
        .arg("--since".to_string())
        .arg("@".to_string() + &since.to_string())
        .arg("--until".to_string())
        .arg("@".to_string() + &until.to_string())
        .output();

    let mut vect = Vec::new();

    if let SudoExecutionOutput::Success(output) = journalctl_command {
        let o = output.stdout;
        for l in o.lines() {
            match serde_json::from_str::<JournalEntryDeserializationSchema>(l) {
                Ok(parsed_entry) => {
                    let mut user: Option<NativeUser> = None;

                    if let Some(uid_field) = parsed_entry.uid {
                        if let Ok(numeric_uid) = uid_field.parse::<u32>() {
                            if let Ok(found_user) = NativeUser::from_uid(numeric_uid) {
                                user = Some(found_user)
                            }
                        }
                    }

                    if user.is_none() {
                        if let Some(user_field) = parsed_entry.user {
                            if let Ok(found_user) = NativeUser::from_username(user_field) {
                                user = Some(found_user)
                            }
                        }
                    }

                    if user.is_none() {
                        if let Some(username_field) = parsed_entry.username {
                            if let Ok(found_user) = NativeUser::from_username(username_field) {
                                user = Some(found_user)
                            }
                        }
                    }

                    vect.push(JournalEntry {
                        timestamp: parsed_entry.realtime_timestamp.parse::<u128>().unwrap(),
                        message: {
                            if let Some(msg) = parsed_entry.message {
                                Some(msg.to_string())
                            } else {
                                None
                            }
                        },
                        priority: parsed_entry.priority,
                        application: parsed_entry.application,
                        user,
                    })
                }
                Err(_) => {
                    return Err(LogMessageError::BadJournalEntryStructure());
                }
            }
        }
    } else {
        return Err(LogMessageError::InvocationError);
    }

    Ok(vect)
}

// TODO Write some tests
