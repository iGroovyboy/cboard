#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{fs, thread, thread::sleep, time::Duration};
use std::sync::{Arc, Mutex};

use arboard::Clipboard;
use inputbot::KeybdKey::*;
use tauri::{AppHandle, Manager};

use app::{APP_HANDLE, ClipboardContent, FileTypes, my_clipboard};

mod tray;
mod helpers;
mod filesys;
mod window;
mod clipboard;

struct ClipboardPreviousText {
    text: Arc<Mutex<String>>,
}

#[derive(Debug, serde::Deserialize)]
struct ClipboardItem {
    name: String,
    folder: String,
    path: String,
    contents: Option<String>,
}


#[tauri::command]
fn enable_clipboard() -> Result<String, String> {
    // let stateClone = Arc::clone(&state.text);
    // let mut stateText = stateClone.lock().unwrap();

    // *stateText = "wasd".to_string(); // TODO: load init here?

    println!("Clipboard management was enabled!");

    // TODO: move to mod
    // image can get to clipboard in many ways, so we use interval-based checker
    thread::spawn(move || {
        my_clipboard::image::init_prev_image().unwrap();

        loop {
            sleep(Duration::from_millis(1000));

            if !my_clipboard::has_image() {
                continue;
            }

            // TODO: will get clipboard and prev_image instances
            // TODO: every cycle - is it ok?
            my_clipboard::image::on_copy();
        }
    });

    // TODO: move to mod
    // TODO: test/fix bind on linux/mac
    // event listener: Ctrl + C
    CKey.bind(move || {
        if LControlKey.is_pressed() {
            // before sleep we get access to prev clipboard data
            sleep(Duration::from_millis(10));

            // here we have just recently clipboard data
            let clipboard = my_clipboard::get_instance();
            let mut clipboard_lock = clipboard.lock().unwrap();

            match clipboard_lock.get_text() {
                Ok(text) => {
                    let mut previous_text = my_clipboard::text::get_previous_text().unwrap();
                    match previous_text {
                        None => {
                            my_clipboard::text::set_previous_text(text.clone());
                            my_clipboard::save_contents(ClipboardContent::Text(text));
                        }
                        Some(prev_text) => {
                            if text != prev_text {
                                my_clipboard::save_contents(
                                    ClipboardContent::Text(text)
                                );
                            }
                        }
                    }
                }
                Err(_) => {
                    my_clipboard::image::on_copy();
                }
            }
        }
    });

    inputbot::handle_input_events();

    return Ok(123.to_string());
}


// fn send(event_type: &EventType) {
//   let delay = Duration::from_millis(20);
//   match simulate(event_type) {
//       Ok(()) => (),
//       Err(SimulateError) => {
//           println!("We could not send {:?}", event_type);
//       }
//   }
//   // Let ths OS catchup (at least MacOS)
//   thread::sleep(delay);
// }

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

    println!("paste -> {:#?}", from.extension().unwrap());

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
    clipboard.clear();
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
            APP_HANDLE.set(handle).unwrap_or_else(|_| panic!("AppHandle is already set"));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
      enable_clipboard, 
      filesys::remove_clipboard_item,
      filesys::move_clipboard_item,
      paste,
      filesys::deleteAllByFolder,
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