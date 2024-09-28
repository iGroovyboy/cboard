use crate::{
    autorun::autorun,
    filesys::{read_json_data, FILENAME_SETTINGS},
    hotkeys_listener,
};
use parking_lot::Mutex;
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::{Arc, OnceLock};
use crate::filesys::write_json_data;

pub static DEFAULT_MAX_CLIPBOARD_ITEMS: u16 = 150;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub clipboard_max_count: u16,
    pub autorun: bool,
    pub win_key: WinKeySetting,
    pub win_key_hotkey: String,
    pub show_app_hotkey: String,
}

pub static SETTINGS: OnceLock<Arc<Mutex<Settings>>> = OnceLock::new();

#[derive(Debug, Copy, Clone, Serialize)]
pub enum WinKeySetting {
    Normal = 0,
    DisableInFullscreen = 1,
    Hotkey = 2,
}

impl TryFrom<i8> for WinKeySetting {
    type Error = &'static str;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WinKeySetting::Normal),
            1 => Ok(WinKeySetting::DisableInFullscreen),
            2 => Ok(WinKeySetting::Hotkey),
            _ => Err("Invalid value for WinKeySetting"),
        }
    }
}

impl<'de> Deserialize<'de> for WinKeySetting {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = i8::deserialize(deserializer)?;
        WinKeySetting::try_from(value).map_err(serde::de::Error::custom)
    }
}

pub fn get_settings_instance() -> Arc<parking_lot::Mutex<Settings>> {
    SETTINGS
        .get_or_init(|| {
            Arc::new(parking_lot::Mutex::new(Settings {
                clipboard_max_count: DEFAULT_MAX_CLIPBOARD_ITEMS,
                autorun: false,
                win_key: WinKeySetting::Normal,
                win_key_hotkey: "".to_string(),
                show_app_hotkey: "LControl,Key1".to_string(),
            }))
        })
        .clone()
}

fn set_settings(new_data: Option<Settings>) -> Settings {
    let settings = get_settings_instance();
    let mut settings = settings.lock();

    if let Some(data) = new_data {
        *settings = data.clone();
    }

    settings.clone()
}

// TODO: create global event
#[allow(dead_code)]
#[tauri::command]
pub fn update_settings() -> Result<(), String> {
    match read_json_data::<Settings>(FILENAME_SETTINGS) {
        Ok(data) => {
            let settings = set_settings(Some(data));

            // TODO: move out to global event listener
            autorun(settings.autorun);
            hotkeys_listener::run();

            Ok(())
        }
        Err(_) => {
            let default_settings = set_settings(None);
            write_json_data(FILENAME_SETTINGS, &default_settings);

            // TODO: move out to global event listener
            autorun(default_settings.autorun);
            hotkeys_listener::run();

            Ok(())
        }
    }
}
