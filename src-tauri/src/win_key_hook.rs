use crate::hotkeys_listener::parse_keycodes;
use crate::keys::{hotkeys_list_devicequery_to_rdev, send_hotkeys, send_string};
use crate::processes::{app_active_state, is_fg_window_fullscreen};
use crate::settings::{get_settings_instance, update_settings, WinKeySetting};
use std::ptr;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, HC_ACTION, KBDLLHOOKSTRUCT,
    MSG, VK_LWIN, VK_RWIN, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

unsafe extern "system" fn hook_callback(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if code == HC_ACTION && app_active_state() {
        let kb_struct = *(l_param as *const KBDLLHOOKSTRUCT);

        if (kb_struct.vkCode == VK_LWIN as u32 || kb_struct.vkCode == VK_RWIN as u32)
            && (w_param == WM_KEYDOWN as WPARAM || w_param == WM_SYSKEYDOWN as WPARAM)
        {
            let allow_win_key = 0;
            let supress_win_key = 1;

            if !app_active_state() {
                return allow_win_key;
            }

            let settings = get_settings_instance();
            let settings = settings.lock();

            match settings.win_key {
                WinKeySetting::DisableInFullscreen => {
                    if is_fg_window_fullscreen() {
                        return supress_win_key;
                    } else {
                        return allow_win_key;
                    }
                }
                WinKeySetting::Hotkey => {
                    match parse_keycodes(settings.win_key_hotkey.clone()) {
                        Ok(hotkeys) => {
                            let data = hotkeys_list_devicequery_to_rdev(hotkeys);
                            send_hotkeys(data);
                        }
                        Err(err) => println!(
                            "Error parsing hotkeys {:#?}: {}",
                            settings.win_key_hotkey, err
                        ),
                    }

                    return supress_win_key;
                }
                WinKeySetting::Text => {
                    let _ = send_string(settings.win_key_text.as_str());
                    return supress_win_key;
                }
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

    // TODO: remove loop?
    loop {
        let mut msg: MSG = std::mem::zeroed();
        GetMessageW(&mut msg, ptr::null_mut(), 0, 0);
        DispatchMessageW(&msg);
    }
}
