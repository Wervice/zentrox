use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::process::Command;
use std::thread;

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct BlockDevice {
    name: String,
    mountpoint: String,
    children: Option<Vec<BlockDevice>>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct LsblkOutput {
    blockdevices: Vec<BlockDevice>,
}

// Lsblk is unfinished

pub fn device_list() -> LsblkOutput {
    let lsblk_output = Command::new("lsblk")
        .arg("-o")
        .arg("NAME,MOUNTPOINT")
        .arg("--bytes")
        .arg("--json")
        .output()
        .unwrap()
        .stdout;
    
    let json = String::from_utf8_lossy(&lsblk_output).to_string();

    println!("{}", &json);

    let o: LsblkOutput =
        serde_json::from_str(&json).unwrap();
    o
}

pub fn device_information(device_name: String) -> Option<BlockDevice> {
    let lsblk_output = Command::new("lsblk")
        .arg("-o")
        .arg("--bytes")
        .arg("--json")
        .output()
        .unwrap()
        .stdout;

    let o: LsblkOutput = 
        serde_json::from_str(&std::str::from_utf8(&lsblk_output).unwrap_or("").to_string()).unwrap();
    
    for e in o.blockdevices {
        if e.name == device_name {
            return Some(e)
        }
    }

    return None
}
