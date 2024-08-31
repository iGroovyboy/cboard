use std::sync::{Arc, OnceLock};
use serde::Deserialize;
use crate::{filesys::{read_json_data, FILENAME_SETTINGS}};
use parking_lot::Mutex;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Settings {
    autorun: bool,
    win_key: String,
    win_key_hotkey: String,
    show_app_hotkey: String,
}

pub static SETTINGS: OnceLock<Arc<Mutex<Settings>>> = OnceLock::new();

fn get_settings_instance() -> Arc<parking_lot::Mutex<Settings>> {
    SETTINGS.get_or_init(|| {
        Arc::new(parking_lot::Mutex::new(
        Settings { 
                autorun: true, 
                win_key: "0".to_string(), 
                win_key_hotkey: "".to_string(), 
                show_app_hotkey: "LControl,Key1".to_string(), 
            }))
    }).clone()
}

fn set_settings(new_data: Settings) {
    let settings = get_settings_instance();
    let mut settings = settings.lock();

    println!("SETTINGS! {:#?}", new_data);

    *settings = new_data;

}

#[allow(dead_code)]
#[tauri::command]
pub fn update_settings() -> Result<(), String> {
    match read_json_data::<Settings>(FILENAME_SETTINGS) {
        Ok(data) => {
            set_settings(data);
            Ok(())
        }
        Err(_) => {
            eprintln!("Failed to read JSON: {}", FILENAME_SETTINGS);
            Ok(())
        }
    }
}