#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::thread;
use tauri::{Manager};
use app::helpers::APP_HANDLE;
use app::{auto_replacement, clipboard as my_clipboard, filesys, hotkeys_reader, processes, settings, tray, win_key_hook, window};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }

            let handle = app.handle().clone();
            APP_HANDLE
                .set(handle)
                .unwrap_or_else(|_| panic!("AppHandle is already set"));

            auto_replacement::enable_key_listener();

            thread::spawn(|| unsafe {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(processes::watch_active_window());
            });

            thread::spawn(|| unsafe {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(win_key_hook::win_key_hook());
            });


            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            filesys::remove_clipboard_item,
            filesys::move_clipboard_item,
            filesys::delete_all_by_folder,
            filesys::read_clipboard_data,
            window::hide_window,
            window::show_window,
            window::quit,
            my_clipboard::enable_clipboard,
            my_clipboard::paste,
            auto_replacement::update_auto_replace_data,
            processes::get_proccesses_list,
            processes::update_blacklist_data,
            hotkeys_reader::hotkeys_listen,
            hotkeys_reader::hotkeys_unlisten,
            settings::update_settings,
        ])
        .system_tray(tray::make_tray())
        .on_system_tray_event(tray::handle_tray_events)
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
