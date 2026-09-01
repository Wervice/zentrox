use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{
    Local, TimeZone,
    prelude::{DateTime, Utc},
};

/// Gets the milliseconds since 1/1/1970 UTC+0.
pub fn current_timestamp_unix() -> u128 {
    chrono::Local::now().timestamp_millis() as u128
}

pub fn current_timestamp_iso() -> String {
    let datetime: DateTime<Utc> = current().into();
    datetime.format("%+").to_string()
}

pub fn current() -> DateTime<Local> {
    chrono::Local::now()
}

pub fn time_to_unix(s: SystemTime) -> u128 {
    s.duration_since(UNIX_EPOCH).unwrap().as_millis()
}

pub fn get_utc_offset() -> i32 {
    Local
        .timestamp_opt(0, 0)
        .unwrap()
        .offset()
        .local_minus_utc()
}
