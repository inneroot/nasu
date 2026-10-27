use chrono::{DateTime, Local, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_timestamp() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    since_the_epoch.as_millis()
}

pub fn format_timestamp(timestamp: u128) -> String {
    let seconds = (timestamp / 1000) as i64;
    let nanos = ((timestamp % 1000) * 1_000_000) as u32;
    let utc = DateTime::<Utc>::from_timestamp(seconds, nanos);

    match utc {
        Some(dt) => dt
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S UTC%:z")
            .to_string(),
        None => current_timestamp().to_string(),
    }
}