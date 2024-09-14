use regex::Regex;
use serde::Deserialize;
use serde_json;
use std::process::Command;

#[derive(Deserialize, serde::Serialize)]
pub struct BlockDevice {
    pub name: String,
    pub mountpoint: Option<String>,
    pub children: Option<Vec<BlockDevice>>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct LsblkOutput {
    pub blockdevices: Vec<BlockDevice>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct Ussage {
    pub filesystem: String,
    pub size: u64,
    pub used: u64,
    pub available: u64,
    pub capacity: u64,
    pub mounted: String,
}

#[derive(Deserialize, serde::Serialize, Clone)]
pub struct Drive {
    model: Option<String>,
    path: Option<String>,
    size: Option<u64>,
    owner: Option<String>,
    mountpoint: Option<String>,
    fsused: Option<f64>,
    name: Option<String>,
    children: Option<Vec<Drive>>,
}

#[derive(Deserialize, serde::Serialize)]
pub struct LsblkOutputExhaustive {
    pub blockdevices: Vec<Drive>,
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
            println!("❌ {}", e);
            None
        }
    }
}

/// Return drive statistics about a drive.
/// * `drive` - The drive name
/// This function returns every entry where the specefied drive name is in the path.
pub fn drive_statistics(drive: String) -> Option<Vec<(String, u64, u64, u64, f64, String)>> {
    let dfp_output = Command::new("df").arg("-P").output().unwrap().stdout;
    let re = Regex::new(r"\s+").unwrap();

    let dfp_output_lines: Vec<String> = String::from_utf8_lossy(&dfp_output)
        .to_string()
        .lines()
        .skip(1)
        .map(|x| x.to_string())
        .collect();

    let mut ussage_vector: Vec<(String, u64, u64, u64, f64, String)> = Vec::new();

    for line in dfp_output_lines {
        let dfp_output_s = re.split(&line);
        let dfp_output_split = dfp_output_s.collect::<Vec<&str>>();
        if dfp_output_split[0].contains(&drive) {
            ussage_vector.push((
                dfp_output_split[0].to_string(),
                dfp_output_split[1].to_string().parse().unwrap(),
                dfp_output_split[2].to_string().parse().unwrap(),
                dfp_output_split[3].to_string().parse().unwrap(),
                dfp_output_split[4]
                    .to_string()
                    .replace("%", "")
                    .to_string()
                    .parse()
                    .unwrap(),
                dfp_output_split[5].to_string(),
            ))
        }
    }

    return Some(ussage_vector);
}

pub fn drive_information(device_name: String) -> Option<Drive> {
    let mut binding = Command::new("lsblk");
    let c = binding
        .arg("--bytes")
        .arg("--json")
        .arg("-o")
        .arg("NAME,MODEL,PATH,SIZE,OWNER,MOUNTPOINT,FSUSED");
    let c_output = match c.output() {
        Ok(v) => v.stdout,
        Err(e) => {
            eprintln!("❌ Failed to spawn lsblk --bytes --json -o NAME,MODEL,PATH,SIZE,OWNER,MOUNTPOINT,FSUSED when getting information about a specific drive.\n{}", e);
            return None;
        }
    };

    let c_output_s = String::from_utf8_lossy(&c_output).to_string();

    let o: LsblkOutputExhaustive =
        serde_json::from_str(&c_output_s).expect("❌ Parsing Lsblk failed");

    for bkdv in o.blockdevices {
        if *bkdv.name.as_ref().unwrap() == device_name {
            return Some(bkdv);
        } else if bkdv.children.is_some() {
            for child in &mut bkdv.children.unwrap() {
                if child.name.clone().unwrap() == device_name {
                    return Some(child.clone());
                }
            }
        }
    }

    None
}
