use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Debug)]
pub enum UptimeError {
    ReadError,
    BadData,
    ParseError,
}

/// Provides the seconds since last boot.
/// This may return an error if the value of `/proc/uptime` is malformed or could not be read.
pub fn get() -> Result<Duration, UptimeError> {
    if let Ok(v) = fs::read_to_string(std::path::Path::new("/proc/uptime")) {
        let seg: Vec<&str> = v.split(' ').collect();
        if seg.len() < 2 {
            Err(UptimeError::BadData)
        } else {
            let v = seg[0];
            let parsed = v.parse::<f32>();
            match parsed {
                Ok(v) => Ok(Duration::from_secs_f32(v)),
                Err(_) => Err(UptimeError::ParseError),
            }
        }
    } else {
        Err(UptimeError::ReadError)
    }
}

/// Struct capturing parts of the contents of /proc/loadavg.
///
/// This struct has the same fields as [`api::dashboard::Load`]. In order to not completely
/// intertwine backend APIs and front-end exchange, these structs are separated into two files.
pub struct Load {
    /// The load averages as defined in man proc_loadavg(5) from 1, 5 and 15 minutes ago
    pub averages: (f32, f32, f32),
    pub latest_process: u32,
}

/// Retrieves
pub fn load_average() -> Result<Load, UptimeError> {
    if let Ok(v) = fs::read_to_string(Path::new("/proc/loadavg")) {
        let mut v_trunc = v;
        v_trunc.pop();
        let seg: Vec<&str> = v_trunc.split(' ').collect();
        if seg.len() != 5 {
            Err(UptimeError::BadData)
        } else {
            let averages: (f32, f32, f32) = (
                seg[0].parse().unwrap(),
                seg[1].parse().unwrap(),
                seg[2].parse().unwrap(),
            );
            let latest_process: u32 = seg[4].parse().unwrap();
            Ok(Load {
                averages,
                latest_process,
            })
        }
    } else {
        Err(UptimeError::ReadError)
    }
}
