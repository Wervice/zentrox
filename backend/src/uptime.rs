use std::fs;
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
    let read = fs::read_to_string(std::path::Path::new("/proc/uptime"));
    let d = match read {
        Ok(v) => v,
        Err(_) => return Err(UptimeError::ReadError),
    };

    let seg: Vec<&str> = d.split(' ').collect();
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
}
