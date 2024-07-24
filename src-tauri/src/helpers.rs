use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
pub fn get_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

pub fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

pub fn is_alphabetic_or_space(s: &str) -> bool {
    s.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
}
