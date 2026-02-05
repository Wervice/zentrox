use chrono::prelude::{DateTime, Utc};
use std::time::{self, SystemTime, UNIX_EPOCH};

pub fn current_timestamp_unix() -> u128 {
    time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time moved backwards.")
        .as_millis()
}

pub fn time_to_unix(time: SystemTime) -> u128 {
    time.duration_since(UNIX_EPOCH)
        .expect("Time went backwards.")
        .as_millis()
}

pub fn current_timestamp_iso() -> String {
    let datetime: DateTime<Utc> = current().into();
    datetime.format("%+").to_string()
}

pub fn current() -> SystemTime {
    time::SystemTime::now()
}
