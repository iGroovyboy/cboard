use core::time;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::{ptr, thread};
use std::sync::{Arc, OnceLock};
use parking_lot::Mutex;
use winapi::ctypes::c_int;
use winapi::shared::minwindef::{HKL, LOWORD};
use winapi::um::winnls::{GetLocaleInfoW, LCIDToLocaleName};
use winapi::um::winnt::{LOCALE_NAME_MAX_LENGTH};
use winapi::um::winuser::{GetKeyboardLayoutList, GetWindowThreadProcessId};
use winapi::um::winuser::{GetForegroundWindow, GetKeyboardLayout};
use winapi::um::winnt::LANGID;
use winapi::shared::windef::HWND;
use winapi::um::winuser::LoadKeyboardLayoutW;
use winapi::um::winuser::KLF_ACTIVATE;
use winapi::um::winuser::PostMessageW;
use winapi::um::winuser::WM_INPUTLANGCHANGEREQUEST;
use winapi::shared::minwindef::LPARAM;
use crate::helpers;
use serde::{Deserialize, Serialize};
use crate::filesys::{read_json_data, write_json_data, FILENAME_KEYBOARD_LAYOUTS};
use crate::processes::BlacklistItem;

/// TODO: add linux/macos

pub const LOCALE_SNATIVELANGNAME: u32 = 4;
pub const LOCALE_SENGLANGUAGE: u32 = 4097;

#[derive(Debug, Serialize)]
pub struct KeyboardLayout {
    pub lang_id: LANGID,
    pub lang_code: String,
    pub lang_name: String,
}

impl KeyboardLayout {
    pub fn from_handle(hkl: u16) -> Self {
        KeyboardLayout {
            lang_id: hkl,
            lang_code: Self::langid_to_locale(hkl as u16),
            lang_name: Self::langid_to_lang_name(hkl as u16),
        }
    }

    // 1033 -> "en-US"
    fn langid_to_locale(langid: LANGID) -> String {
        let mut buffer = [0u16; LOCALE_NAME_MAX_LENGTH as usize];
        let result = unsafe {
            LCIDToLocaleName(
                langid as u32,//MAKELCID(langid, 0),
                buffer.as_mut_ptr(),
                LOCALE_NAME_MAX_LENGTH as i32,
                0,
            )
        };

        Self::convertResultToString(&mut buffer, &result)
    }

    // 1033 -> "English"
    fn langid_to_lang_name(langid: LANGID) -> String {
        let mut buffer = [0u16; LOCALE_NAME_MAX_LENGTH as usize];
        let result = unsafe {
            GetLocaleInfoW(
                langid as u32,
                LOCALE_SNATIVELANGNAME, //LOCALE_SENGLANGUAGE,
                buffer.as_mut_ptr(),
                LOCALE_NAME_MAX_LENGTH as i32,
            )
        };

        Self::convertResultToString(&mut buffer, &result)
    }

    fn convertResultToString(buffer: &mut [u16; LOCALE_NAME_MAX_LENGTH as usize], result: &c_int) -> String {
        if *result > 0 {
            let os_string = OsString::from_wide(&buffer[..*result as usize - 1]);
            os_string.to_string_lossy().into_owned()
        } else {
            "Unknown".to_string()
        }
    }
}

// -> 1033
pub fn get_current_keyboard_lang_id() -> u16 {
    unsafe {
        let foreground_window = GetForegroundWindow();
        let thread_id = GetWindowThreadProcessId(foreground_window, ptr::null_mut());
        let layout_id = GetKeyboardLayout(thread_id);
        LOWORD(layout_id as u32) as u16
    }
}

// -> "en-US"
pub fn get_current_keyboard_layout_locale() -> String {
    let lang_id = unsafe { get_current_keyboard_lang_id() };
    KeyboardLayout::langid_to_locale(lang_id)
}

#[tauri::command]
pub fn get_available_keyboard_layouts() -> Result<Vec<KeyboardLayout>, String> {
    let count = unsafe { GetKeyboardLayoutList(0, ptr::null_mut()) };
    if count == 0 {
        return Ok(vec![]);
    }

    let mut layouts: Vec<HKL> = vec![ptr::null_mut(); count as usize];
    let ret = unsafe { GetKeyboardLayoutList(count, layouts.as_mut_ptr()) };
    if ret == 0 {
        eprintln!("Failed to get keyboard layouts");
        return Ok(vec![]);
    }

    Ok(layouts
        .into_iter()
        .map(|handle| KeyboardLayout::from_handle(handle as u16))
        .collect())
}

pub fn change_keyboard_layout(window_handle: HWND, lang_code: u16) {
    if lang_code == get_current_keyboard_lang_id() {
        return;
    }

    let hex = format!("{:08X}", lang_code);
    let layout: Vec<_> = helpers::to_wide_string(hex.as_str());

    let hkl = unsafe { LoadKeyboardLayoutW(layout.as_ptr(), KLF_ACTIVATE) };

    if hkl.is_null() {
        eprintln!("Failed to load keyboard layout");
        return;
    }

    unsafe { PostMessageW(window_handle, WM_INPUTLANGCHANGEREQUEST, 0, hkl as LPARAM); }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppItem {
    pub enabled: bool,
    pub filename: Option<String>,
    pub filepath: String,
    pub lang_id: u16,
    pub title: Option<String>,
}

// TODO: same as processes.rs::BLACKILST, refactor
pub static KEY_APPS_LIST: OnceLock<Arc<Mutex<Vec<AppItem>>>> = OnceLock::new();

pub fn get_key_apps_instance() -> Arc<parking_lot::Mutex<Vec<AppItem>>> {
    KEY_APPS_LIST
        .get_or_init(|| Arc::new(parking_lot::Mutex::new(vec![])))
        .clone()
}

pub fn is_key_apps_empty() -> bool {
    let key_apps = get_key_apps_instance();
    let key_apps = key_apps.lock();
    key_apps.is_empty()
}

fn set_key_apps_data(new_data: Option<Vec<AppItem>>) -> Vec<AppItem> {
    let key_apps = get_key_apps_instance();
    let mut key_apps = key_apps.lock();

    if let Some(data) = new_data {
        *key_apps = data.clone();
    }

    key_apps.clone()
}

#[tauri::command]
pub fn update_keyboard_layouts_data() -> Result<(), String> {
    match read_json_data::<Vec<AppItem>>(FILENAME_KEYBOARD_LAYOUTS) {
        Ok(data) => {
            set_key_apps_data(Some(data));

            Ok(())
        }
        Err(_) => {
            let default_settings = set_key_apps_data(None);
            write_json_data(FILENAME_KEYBOARD_LAYOUTS, &default_settings);

            Ok(())
        }
    }
}