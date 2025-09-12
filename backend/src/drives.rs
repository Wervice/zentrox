use log::warn;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, process::Command};
use utoipa::{ToResponse, ToSchema};

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct BlockDevice {
    pub name: String,
    pub mountpoint: Option<String>,
    #[schema(value_type = Option<Vec<Object>>)]
    pub children: Option<Vec<BlockDevice>>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct LsblkOutput {
    pub blockdevices: Vec<BlockDevice>,
}

#[derive(Deserialize, serde::Serialize, ToSchema, ToResponse)]
pub struct Usage {
    pub filesystem: String,
    pub size: u64,
    pub used: u64,
    pub available: u64,
    pub capacity: u64,
    pub mounted: String,
}

#[derive(Deserialize, serde::Serialize, Clone, Debug, ToSchema)]
pub struct Drive {
    model: Option<String>,
    path: Option<String>,
    size: Option<u64>,
    owner: Option<String>,
    mountpoint: Option<String>,
    fsused: Option<f64>,
    name: Option<String>,
    #[schema(value_type = Option<Vec<Object>>)]
    children: Option<Vec<Drive>>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct LsblkOutputExhaustive {
    pub blockdevices: Vec<Drive>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DriveUsageStatistics {
    pub filesystem: String,
    #[schema(value_type = String)]
    pub mountpoint: PathBuf,
    pub capacity: f32,
    pub free: u32,
    pub total: u32,
    pub in_use: u32,
}

/// List all block device on the system.
pub fn device_list() -> Option<LsblkOutput> {
    let lsblk_output = Command::new("lsblk")
        .arg("-o")
        .arg("NAME,MOUNTPOINT")
        .arg("--bytes")
        .arg("--json")
        .output()
        .unwrap()
        .stdout;

    let json = String::from_utf8_lossy(&lsblk_output).to_string();

    match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(e) => {
            println!("❌ {e}");
            None
        }
    }
}

/// Return drive statistics about a drive.
/// * `drive` - The drive name
///
/// This function returns every entry where the specified drive name is in the path.
pub fn drive_statistics(drive: String) -> Option<DriveUsageStatistics> {
    let dfp_output = Command::new("df").arg("-P").output().unwrap().stdout;
    let re = Regex::new(r"\s+").unwrap();

    let dfp_output_lines: Vec<String> = String::from_utf8_lossy(&dfp_output)
        .to_string()
        .lines()
        .skip(1)
        .map(|x| x.to_string())
        .collect();

    let mut statistics: Option<DriveUsageStatistics> = None;

    for line in dfp_output_lines {
        let dfp_output_s = re.split(&line);
        let dfp_output_split = dfp_output_s.collect::<Vec<&str>>();
        if dfp_output_split[0].contains(&drive) {
            let total = dfp_output_split[1].parse::<u32>().unwrap() * 1024;
            let in_use = dfp_output_split[2].parse::<u32>().unwrap() * 1024;
            let free = dfp_output_split[3].parse::<u32>().unwrap() * 1024;
            statistics = Some(DriveUsageStatistics {
                filesystem: dfp_output_split[0].to_string(),
                mountpoint: PathBuf::from(dfp_output_split[5]),
                total,
                free,
                in_use,
                capacity: in_use as f32 / total as f32,
            });
            break;
        }
    }

    statistics
}

/// Get information about a specified block device
pub fn drive_information(device_name: String) -> Option<Drive> {
    // NOTE Return Result<Drive>
    // instead of Option<>
    let mut binding = Command::new("lsblk");
    let c = binding
        .arg("--bytes")
        .arg("--json")
        .arg("-o")
        .arg("NAME,MODEL,PATH,SIZE,OWNER,MOUNTPOINT,FSUSED");
    let c_output = match c.output() {
        Ok(v) => v.stdout,
        Err(e) => {
            warn!("Failed to spawn command to view blockdevice metadata.");
            return None;
        }
    };

    let c_output_s = String::from_utf8_lossy(&c_output).to_string();

    let o: LsblkOutputExhaustive =
        serde_json::from_str(&c_output_s).expect("❌ Parsing Lsblk failed");

    fn scan(o: Vec<Drive>, device_name: String) -> Option<Drive> {
        for dev in o {
            if dev.clone().name.unwrap() == device_name {
                return Some(dev.clone());
            } else if dev.children.is_some() {
                return scan(dev.children.unwrap(), device_name);
            }
        }
        None
    }

    scan(o.blockdevices, device_name)
}
