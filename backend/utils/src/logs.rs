use std::{fmt::Display, time::{Duration}};
use crate::sudo::SudoCommand;
use crate::users::NativeUser;
use log::{debug, error};
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
    pub realtime_timestamp: PhantomStringOrDigit,

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

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum PhantomStringOrDigit {
    String(String),
    Digit(u128)
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
/// JournalEntry discarding user details and only using a numeric reference.
/// This saves time when sending the serialized form over JSON.
pub struct QuickJournalEntry {
    pub timestamp: u128,
    pub message: Option<String>,
    pub priority: Option<String>,
    pub user: Option<u32>,
    pub application: Option<String>,
}

impl JournalEntry {
    pub fn as_quick_journal_entry(self) -> QuickJournalEntry {
        QuickJournalEntry {
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
        }
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
    since: Duration,
    until: Duration,
) -> Result<Vec<JournalEntry>, LogMessageError> {
    debug!("Getting log messages between: {:?} and {:?}", since, until);
    let journalctl_command = SudoCommand::new(sudo_password, "journalctl".to_string())
        .args(vec![
            "-o",
            "json",
            "--since",
            format!("@{}", since.as_secs()).as_str(),
            "--until",
            format!("@{}", until.as_secs()).as_str(),
        ])
        .output();

    if let Ok(output) = journalctl_command {
        let o = output.stdout;
        o.lines()
            .map(|line| {
                let parse = serde_json::from_str::<JournalEntryDeserializationSchema>(line);
                if let Ok(parsed_entry) =
                    parse
                {
                    let user: Option<NativeUser> = if let Some(uid_field) = parsed_entry.uid
                        && let Ok(numeric_uid) = uid_field.parse::<u32>()
                        && let Ok(found_user) = NativeUser::from_uid(numeric_uid)
                    {
                        Some(found_user)
                    } else if let Some(user_field) = parsed_entry.user
                        && let Ok(found_user) = NativeUser::from_username(user_field)
                    {
                        Some(found_user)
                    } else if let Some(username_field) = parsed_entry.username
                        && let Ok(found_user) = NativeUser::from_username(username_field)
                    {
                        Some(found_user)
                    } else {
                        None
                    };

                    Ok(JournalEntry {
                        timestamp: {
                            match parsed_entry.realtime_timestamp {
                                PhantomStringOrDigit::Digit(d) => d,
                                PhantomStringOrDigit::String(s) => s.parse::<u128>().unwrap()
                            }
                        },
                        message: parsed_entry.message.map(|msg| msg.to_string()),
                        priority: parsed_entry.priority,
                        application: parsed_entry.application,
                        user,
                    })
                } else {
                    error!("Failed to parse journalctl entry.");
                    Err(LogMessageError::BadJournalEntryStructure())
                }
            })
            .collect()
    } else {
        Err(LogMessageError::InvocationError)
    }
}
