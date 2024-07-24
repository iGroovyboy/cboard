#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{fs, thread, thread::sleep, time::Duration};

use arboard::Clipboard;
use inputbot::KeybdKey::*;
use tauri::{AppHandle, Manager};

use app::{APP_HANDLE, FileTypes, my_clipboard};

mod filesys;
mod helpers;
mod input_lang;
mod keys;
mod tray;
mod window;

pub static IMG_THREAD_INTERVAL: u64 = 1000;

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct ClipboardItem {
    name: String,
    folder: String,
    path: String,
    contents: Option<String>,
}

#[tauri::command]
fn enable_clipboard() -> Result<(), String> {
    filesys::create_folders(&[filesys::FOLDER_CLIPBOARD, filesys::FOLDER_FAVOURITES])
        .expect("Couldn't create required directories");

    // image can get to clipboard in many ways, so we use interval-based checker
    thread::spawn(move || {
        my_clipboard::image::init_prev_image().unwrap();

        loop {
            sleep(Duration::from_millis(IMG_THREAD_INTERVAL));

            if !my_clipboard::has_image() {
                continue;
            }

            // TODO: will get clipboard and prev_image instances every cycle - mb profile it?
            my_clipboard::image::on_copy();
        }
    });

    // TODO: test/fix bind on linux/mac
    // event listener: Ctrl + C
    CKey.bind(move || {
        if LControlKey.is_pressed() {
            // without sleep we get access to prev clipboard data
            sleep(Duration::from_millis(100));

            my_clipboard::text::on_copy();
        }
    });

    inputbot::handle_input_events();

    Ok(())
}

#[tauri::command]
fn paste(item: ClipboardItem, app: AppHandle) {
    // window::hide_window(app);
    // sleep(Duration::from_millis(50));
    let from = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join("data")
        .join(&item.folder)
        .join(&item.name);

    let mut clipboard = Clipboard::new().unwrap();

    // TODO: fix unwraps?
    // TODO: move to separate mods? use patterns?
    match from.extension().unwrap().to_str().unwrap() {
        FileTypes::TXT => {
            let content = fs::read_to_string(from).unwrap();

            clipboard.set_html(&content, Some(&content)).unwrap();
        }
        FileTypes::PNG => {
            let img = image::io::Reader::open(from).unwrap().decode().unwrap();
            let rgba_image = img.to_rgba8();
            let (width, height) = rgba_image.dimensions();
            let bytes = rgba_image.into_raw();
            let image_data = arboard::ImageData {
                width: width as usize,
                height: height as usize,
                bytes: std::borrow::Cow::Owned(bytes),
            };

            clipboard.set_image(image_data).unwrap();
        }
        &_ => {}
    }

    // TODO: move to mod
    LControlKey.press();
    VKey.press();
    VKey.release();
    LControlKey.release();

    sleep(Duration::from_millis(50));
    clipboard.clear().unwrap();
}

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

            keys::enable_key_listener();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            enable_clipboard,
            filesys::remove_clipboard_item,
            filesys::move_clipboard_item,
            paste,
            filesys::delete_all_by_folder,
            window::hide_window,
            window::show_window,
            window::quit,
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
