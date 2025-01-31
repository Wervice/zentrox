// Rust shorthands for linux commands to get information about network configuration on the system
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::process::Command;

pub fn private_ip() -> Result<IpAddr, ()> {
    #[derive(Debug, Serialize, Deserialize)]
    struct Route {
        dst: String,
        gateway: String,
        dev: String,
        prefsrc: String,
        flags: Vec<String>,
        uid: u32,
        cache: Vec<String>,
    }

    let mut ip_c_program = Command::new("ip");
    let ip_c = ip_c_program.arg("-j").arg("route").arg("get").arg("1");
    let ip_c_x = ip_c.output();
    match ip_c_x {
        Ok(v) => {
            let output = v.stdout;
            let mut r = String::new();
            output.into_iter().for_each(|c| {
                r.push(c as char);
            });
            if !v.status.success() {
                return Err(());
            }
            let routes: Vec<Route> = serde_json::from_str(&r).unwrap();
            if routes.len() == 0 {
                Err(())
            } else {
                Ok(routes[0].prefsrc.parse::<IpAddr>().unwrap())
            }
        }
        Err(_) => Err(()),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransmissionStatistics {
    pub bytes: f64,
    pub packets: i64,
    pub errors: i64,
    pub dropped: i64,
    pub over_errors: Option<i64>,
    pub multicast: Option<i64>,
    pub carrier_errors: Option<i64>,
    pub collisions: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interface {
    pub ifindex: i64,
    pub ifname: String,
    pub flags: Vec<String>,
    pub mtu: i64,
    pub qdisc: String,
    pub operstate: String,
    pub linkmode: String,
    pub group: String,
    pub txqlen: Option<i64>,
    pub link_type: String,
    pub address: String,
    pub broadcast: String,
    pub stats64: HashMap<String, TransmissionStatistics>,
    pub altnames: Option<Vec<String>>,
}

pub fn interface_information() -> Result<Vec<Interface>, ()> {
    let mut ip_c_program = Command::new("ip");
    let ip_c = ip_c_program.arg("-j").arg("-s").arg("link").arg("show");
    let ip_c_x = ip_c.output();
    match ip_c_x {
        Ok(v) => {
            let output = v.stdout;
            let mut r = String::new();
            output.into_iter().for_each(|c| {
                r.push(c as char);
            });
            let interfaces: Vec<Interface> = serde_json::from_str(&r).unwrap();
            Ok(interfaces)
        }
        Err(_) => Err(()),
    }
}
