use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
pub fn get_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}