use std::{fs, path::PathBuf};

use log::error;

pub enum UidConversionError {
    FailedToReadPasswd, // Failed to read the /etc/passwd file
    UidNotInPasswd,     // The uid is not in the passwd file
}

/// Convers a given Linux user id (UID) to the corresponding name of the user.
/// It uses the /etc/passwd file to look up this information.
/// If no username is found, an Err(UidNotInPasswd) is returned.
/// If the /etc/passwd file could not be read, an Err(FailedToReadPasswd) is returned.
pub fn convert_uid_to_name(uid: usize) -> Result<String, UidConversionError> {
    let passwd_file_path = PathBuf::from("/etc/passwd");
    let passwd_file_contents = fs::read_to_string(passwd_file_path);
    match passwd_file_contents {
        Ok(v) => {
            let lines = v.lines();
            let mut username: String = "".to_string();
            lines.for_each(|x| {
                if x.split(":").nth(2).unwrap() == uid.to_string() {
                    username = x.split(":").next().unwrap().to_string();
                }
            });
            if username.is_empty() {
                error!("Did not find any username for the uid {uid}");
                Err(UidConversionError::UidNotInPasswd)
            } else {
                Ok(username)
            }
        }
        Err(e) => {
            error!("Failed to read /etc/passwd because of error {e}");
            Err(UidConversionError::FailedToReadPasswd)
        }
    }
}
