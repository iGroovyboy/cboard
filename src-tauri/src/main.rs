#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod tray;
mod helpers;
mod filesys;
mod window;
mod clipboard;

use inputbot::{KeybdKey::*};
use arboard::{Clipboard, ImageData};
use std::{thread, thread::sleep, time::{Duration}, fs};
use std::path::PathBuf;
use tauri::{Manager, AppHandle};
use std::sync::{Arc, Mutex};
use tauri::State;
use app::{ClipboardContent, FileTypes, my_clipboard};

struct ClipboardPreviousText{
  text: Arc<Mutex<String>>
}

#[derive(Debug, serde::Deserialize)]
struct ClipboardItem {
  name: String,
  folder: String,
  path: String,
  contents: Option<String>,
}


#[tauri::command]
fn enable_clipboard(app: tauri::AppHandle, state: State<ClipboardPreviousText>) -> Result<String, String> {
  let stateClone = Arc::clone(&state.text);
  let mut stateText= stateClone.lock().unwrap();
  *stateText = "wasd".to_string(); // TODO: load init here?

  println!("Clipboard management was enabled! {}", *stateText);

  // TODO: fix this
  let app_clone = app.clone();
  let app_clone_img = app.clone();

  // TODO: move to mod
  // image can get to clipboard in many ways, so we use interval-based checker
  thread::spawn(move || {
    let mut prevImage: Option<ImageData> = None;

    loop {
      sleep(Duration::from_millis(1000));

      if !my_clipboard::has_image() {
        continue;
      }

      let mut c = Clipboard::new().unwrap();
      let image_data = c.get_image().unwrap();

      match prevImage {
        None => {
          prevImage = Some(image_data.clone());
          my_clipboard::save_contents(
            ClipboardContent::Image(image_data),
            &app_clone_img
          );
        },
        Some(ref i) => {
          if !my_clipboard::image::eq(&prevImage.clone().unwrap(), &image_data) {
            prevImage = Some(image_data.clone());
            my_clipboard::save_contents(
              ClipboardContent::Image(image_data),
              &app_clone_img
            );
          }
        },
      }

    }
  });

  // TODO: move to mod
  // TODO: test/fix bind on linux/mac
  // event listener: Ctrl + C
  let stateClone = stateClone.clone();
  CKey.bind(move || {
    if LControlKey.is_pressed() {
      // before sleep we get access to prev clipboard data
      sleep(Duration::from_millis(10));

      // here we have just recently clipboard data
      let mut clipboard = Clipboard::new().unwrap();


      // TODO: this will trigger error if user copies image
      match clipboard.get_text() {
        Ok(text) => {
          let mut stateText = stateClone.lock().unwrap();
          if *stateText == clipboard.get_text().unwrap() {
            println!("Ignore: is dupe");
          } else {
            *stateText = clipboard.get_text().unwrap().to_string();
            my_clipboard::save_contents(
              ClipboardContent::Text(clipboard.get_text().unwrap().to_string()),
              &app_clone
            );
          }
        },
        Err(_) => {
          // get image
          println!("TODO: COPY IMAGE")
        },
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
    },
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
    },
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
    .manage(ClipboardPreviousText { text: Default::default() })
    .setup(|app| {
      #[cfg(debug_assertions)] // only include this code on debug builds
      {
        let window = app.get_window("main").unwrap();
        window.open_devtools();
        window.close_devtools();
      }
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