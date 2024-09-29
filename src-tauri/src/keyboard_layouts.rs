use core::time;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::{ptr, thread};
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

/// TODO: add linux/macos

pub const LOCALE_SNATIVELANGNAME: u32 = 4;
pub const LOCALE_SENGLANGUAGE: u32 = 4097;

#[derive(Debug)]
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

// -> "en-US"
pub fn get_current_keyboard_layout() -> String {
    unsafe {
        let foreground_window = GetForegroundWindow();
        let thread_id = GetWindowThreadProcessId(foreground_window, ptr::null_mut());
        let layout_id = GetKeyboardLayout(thread_id);

        let lang_id = LOWORD(layout_id as u32);

        KeyboardLayout::langid_to_locale(lang_id)
    }
}

pub fn get_available_keyboard_layouts() -> Vec<KeyboardLayout> {
    let count = unsafe { GetKeyboardLayoutList(0, ptr::null_mut()) };
    if count == 0 {
        return vec![];
    }

    let mut layouts: Vec<HKL> = vec![ptr::null_mut(); count as usize];
    let ret = unsafe { GetKeyboardLayoutList(count, layouts.as_mut_ptr()) };
    if ret == 0 {
        eprintln!("Failed to get keyboard layouts");
        return vec![];
    }

    layouts
        .into_iter()
        .map(|handle| KeyboardLayout::from_handle(handle as u16))
        .collect()
}

pub fn change_keyboard_layout(window_handle: HWND, lang_code: u16) {
    let hex = format!("{:08X}", lang_code);
    let layout: Vec<_> = helpers::to_wide_string(hex.as_str());

    let hkl = unsafe { LoadKeyboardLayoutW(layout.as_ptr(), KLF_ACTIVATE) };

    if hkl.is_null() {
        eprintln!("Failed to load keyboard layout");
        return;
    }

    unsafe { PostMessageW(window_handle, WM_INPUTLANGCHANGEREQUEST, 0, hkl as LPARAM); }
}
