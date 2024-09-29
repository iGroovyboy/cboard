use serde::Serialize;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;
use std::iter::once;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

#[derive(Clone, Serialize)]
pub struct EmptyPayload;

pub static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn get_tauri_handle() -> &'static AppHandle {
    APP_HANDLE.get().expect("AppHandle is not set")
}

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

pub fn is_alphanumeric_or_space(s: &str) -> bool {
    s.chars().all(|c| c.is_alphanumeric() || c.is_whitespace())
}

pub fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}