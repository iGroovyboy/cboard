use std::{ptr};
use winapi::shared::minwindef::{LRESULT, WPARAM, LPARAM};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL,
    KBDLLHOOKSTRUCT, HC_ACTION, WM_KEYDOWN, WM_SYSKEYDOWN,
    VK_LWIN, VK_RWIN, MSG, GetMessageW, DispatchMessageW
};
use enigo::{Enigo, Settings, Keyboard, Key};
use enigo::Direction::{Press, Release, Click};
use crate::processes::{app_active_state, is_fg_window_fullscreen};
use crate::settings::{get_settings_instance, update_settings, WinKeySetting};

fn handle_win_key() {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    let _ = enigo.key(Key::Control, Press);
    let _ = enigo.key(Key::Unicode('v'), Click);
    let _ = enigo.key(Key::Control, Release);
}

unsafe extern "system" fn hook_callback(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION && app_active_state() {
        let kb_struct = *(l_param as *const KBDLLHOOKSTRUCT);

        if (kb_struct.vkCode == VK_LWIN as u32 || kb_struct.vkCode == VK_RWIN as u32) &&
            (w_param == WM_KEYDOWN as WPARAM || w_param == WM_SYSKEYDOWN as WPARAM) {

            let allow_win_key = 0;
            let supress_win_key = 1;

            if !app_active_state() {
                return allow_win_key;
            }

            let x = get_settings_instance();
            let x = x.lock();

            match x.win_key {
                WinKeySetting::DisableInFullscreen => {
                    if is_fg_window_fullscreen() {
                       return supress_win_key; 
                    } else {
                       return allow_win_key;
                    }
                },
                WinKeySetting::Hotkey => {
                    handle_win_key();
                    return supress_win_key;
                },
                _ => {
                    return allow_win_key;
                }
            }

        }
    }

    CallNextHookEx(ptr::null_mut(), code, w_param, l_param)
}

#[allow(dead_code)]
pub unsafe fn win_key_hook() {
    let h_instance = GetModuleHandleW(ptr::null());
    let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_callback), h_instance, 0);
    let _ = update_settings();

    if hook.is_null() {
        eprintln!("Failed to install win_key hook!");
        return;
    }

    loop {
        let mut msg: MSG = std::mem::zeroed();
        GetMessageW(&mut msg, ptr::null_mut(), 0, 0);
        DispatchMessageW(&msg);
    }

    UnhookWindowsHookEx(hook);
}
