use log::{debug, warn};
use serde::Deserialize;
use std::process::Command;
use utoipa::ToSchema;

#[derive(Debug)]
pub enum DriveError {
    CommandError,
    ParseError,
}

#[derive(Deserialize, serde::Serialize, Clone, Debug, ToSchema)]
pub struct Drive {
    model: Option<String>,
    path: Option<String>,
    size: Option<u64>,
    owner: Option<String>,
    mountpoint: Option<String>,
    fsused: Option<u64>,
    name: Option<String>,
    #[schema(value_type = Option<Vec<Object>>)]
    children: Option<Vec<Drive>>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct LsblkOutputExhaustive {
    pub blockdevices: Vec<Drive>,
}

/// Get information about a specified block device
pub fn list() -> Result<Vec<Drive>, DriveError> {
    debug!("Getting drives.");
    let mut binding = Command::new("lsblk");
    let c = binding
        .arg("--bytes")
        .arg("--json")
        .arg("-o")
        .arg("NAME,MODEL,PATH,SIZE,OWNER,MOUNTPOINT,FSUSED");
    let c_output = match c.output() {
        Ok(v) => v.stdout,
        Err(_e) => {
            warn!("Failed to spawn lsblk command.");
            return Err(DriveError::CommandError);
        }
    };

    let c_output_s = String::from_utf8_lossy(&c_output).to_string();

    if let Ok(o) = serde_json::from_str::<LsblkOutputExhaustive>(&c_output_s) {
        Ok(o.blockdevices)
    } else {
        warn!("Failed to parse lsblk data.");
        Err(DriveError::ParseError)
    }
}
