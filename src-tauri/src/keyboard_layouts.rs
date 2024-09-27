use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use winapi::ctypes::c_int;
use winapi::shared::minwindef::LOWORD;
use winapi::um::winnls::LCIDToLocaleName;
use winapi::um::winnt::{LOCALE_NAME_MAX_LENGTH, MAKELCID, SORT_DEFAULT};
use winapi::um::winuser::GetWindowThreadProcessId;
use winapi::um::winuser::{GetForegroundWindow, GetKeyboardLayout};

/// TODO: add linux/macos
// const KL_NAMELENGTH: usize = 9;

pub fn get_current_keyboard_layout() -> Option<String> {
    unsafe {
        let foreground_window = GetForegroundWindow();
        let thread_id = GetWindowThreadProcessId(foreground_window, ptr::null_mut());
        let layout_id = GetKeyboardLayout(thread_id);

        let lang_id = LOWORD(layout_id as u32);

        let mut buffer: [u16; LOCALE_NAME_MAX_LENGTH as usize] =
            [0; LOCALE_NAME_MAX_LENGTH as usize];

        if LCIDToLocaleName(
            MAKELCID(lang_id, SORT_DEFAULT),
            buffer.as_mut_ptr(),
            LOCALE_NAME_MAX_LENGTH as c_int,
            0,
        ) != 0
        {
            let locale_name: OsString = OsStringExt::from_wide(&buffer);
            //println!("X-->{:#?}", locale_name.to_string_lossy().trim_end_matches('\0').to_string());
            Some(
                locale_name
                    .to_string_lossy()
                    .trim_end_matches('\0')
                    .to_string(),
            )
        } else {
            eprintln!("Failed to get the input language.");
            None
        }
    }
}

// pub fn get_available_keyboard_layouts() -> Vec<String> {
//     unsafe {
//         let mut layouts = vec![0; 10];
//         let count = GetKeyboardLayoutList(layouts.len() as i32, layouts.as_mut_ptr());
//
//         layouts.truncate(count as usize);
//         layouts.into_iter().map(|layout| get_locale_name(layout)).collect()
//     }
// }

// fn get_locale_name(hkl: isize) -> String {
//     unsafe {
//         let mut buffer = [0u16; KL_NAMELENGTH as usize];
//         let result = GetLocaleInfoW(hkl as u32, LOCALE_SENGLANGUAGE, buffer.as_mut_ptr(), buffer.len() as i32);
//
//         if result > 0 {
//             OsString::from_wide(&buffer[..result as usize])
//                 .to_string_lossy()
//                 .into_owned()
//         } else {
//             "Unknown".to_string()
//         }
//     }
// }
