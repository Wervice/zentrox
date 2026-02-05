use std::{fs, path::PathBuf};

use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Default)]
pub struct NativeUser {
    pub username: String,
    pub password: Option<String>,
    pub user_id: u32,
    pub group_id: u32,
    pub gecos: String,
    #[schema(value_type = Option<String>)]
    pub home_directory: Option<PathBuf>,
    pub login_shell: Option<String>,
}

fn parse_passwd_file() -> Result<Vec<NativeUser>, std::io::Error> {
    let passwd_file_path = PathBuf::from("/etc/passwd");
    let passwd_file_contents = fs::read_to_string(passwd_file_path)?;

    let passwd_file_lines = passwd_file_contents.lines();

    Ok(passwd_file_lines
        .map(|line| {
            let line_split: Vec<String> = line.split(":").map(String::from).collect();

            NativeUser {
                username: line_split[0].clone(),
                password: {
                    if line_split[1] == "x" || line_split[1].is_empty() {
                        None
                    } else {
                        Some(line_split[1].clone())
                    }
                },
                user_id: line_split[2].parse::<u32>().unwrap(),
                group_id: line_split[3].parse::<u32>().unwrap(),
                gecos: line_split[4].clone(),
                home_directory: {
                    match &line_split[5] {
                        v if v.is_empty() => None,
                        v => Some(PathBuf::from(v)),
                    }
                },
                login_shell: {
                    match &line_split[6] {
                        v if v.is_empty() => None,
                        v => Some(v.to_string()),
                    }
                },
            }
        })
        .collect::<Vec<NativeUser>>())
}

#[derive(Debug)]
pub enum SearchError {
    PasswdAccessFailed,
    NotFound,
}

impl NativeUser {
    pub fn from_uid(uid: u32) -> Result<NativeUser, SearchError> {
        if let Ok(users) = parse_passwd_file() {
            let search = users.iter().find(|user| user.user_id == uid);

            if let Some(correct_user) = search {
                Ok(correct_user.clone())
            } else {
                Err(SearchError::NotFound)
            }
        } else {
            Err(SearchError::PasswdAccessFailed)
        }
    }

    pub fn from_username(username: String) -> Result<NativeUser, SearchError> {
        if let Ok(users) = parse_passwd_file() {
            let search = users.iter().find(|user| user.username == username);

            if let Some(correct_user) = search {
                Ok(correct_user.clone())
            } else {
                Err(SearchError::NotFound)
            }
        } else {
            Err(SearchError::PasswdAccessFailed)
        }
    }
}
