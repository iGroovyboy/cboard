#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use inputbot::{KeybdKey::*};
use arboard::Clipboard;
use std::{thread, thread::sleep, time::{Duration, SystemTime, UNIX_EPOCH}, fs, path::{Path, PathBuf}};
use tauri::Manager;
// use tauri::AppHandle;


// the payload type must implement `Serialize` and `Clone`.
#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

fn get_timestamp() -> String {
  SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_millis()
      .to_string()
}

// fn data_path(filename: String, folder: String) -> PathBuf {
//   app_handle();

//   let app_dir = app
//     .path_resolver()
//     .app_local_data_dir()
//     .expect("Failed to resolve app local dir");


//   let p = app_dir.as_path()
//     .join("data")
//     .join(folder);

//   return p;
// }

// TODO: check if previous record has same contents as current and skip if they are equal
// TODO: detect/save image
fn save_clipboard(contents: String, is_text: bool, app: &tauri::AppHandle) {
  let app_dir = app
    .path_resolver()
    .app_local_data_dir()
    .expect("Failed to resolve app local dir");


  let p = app_dir.as_path()
    .join("data")
    .join("clipboard");
  
  let f =  p.join([get_timestamp(), ".txt".to_string()].concat());

  println!("save_clipboard: {}", p.display());

  fs::create_dir_all(p).unwrap();
  fs::write(f, &contents).expect("Unable to write file");
  app.emit_all("clipboard", Payload { message: contents }).unwrap();
}

#[tauri::command]
fn remove_clipboard_item(filename: String, folder: String, app: tauri::AppHandle) {
  let file = app
    .path_resolver()
    .app_local_data_dir()
    .expect("Failed to resolve app local dir")
    .as_path()
    .join("data")
    .join(folder)
    .join(filename);

  fs::remove_file(&file);
  println!("removed file {:?}", file);
  app.emit_all("clipboard", Payload { message: "remove_clipboard_item".to_string() }).unwrap();
}

#[tauri::command]
fn move_clipboard_item(from: String, filename: String, folder: String, app: tauri::AppHandle) {
  let to = app
    .path_resolver()
    .app_local_data_dir()
    .expect("Failed to resolve app local dir")
    .as_path()
    .join("data")
    .join(folder)
    .join(&filename);

  fs::rename(from, &to);
  println!("moved file {} to {:?}", &filename, to);
  app.emit_all("clipboard", Payload { message: "move_clipboard_item".to_string() }).unwrap();
}

#[tauri::command]
fn enable_clipboard(app: tauri::AppHandle) -> Result<String, String> {
  println!("Clipboard management was enabled!");

  let app_clone = app.clone();

  thread::spawn(move || {
    let mut i = 1;

    loop {
      if i > 5 {
        break;
      }
      i += 1;
  
      sleep(Duration::from_millis(1000));

      let mut clipboard = Clipboard::new().unwrap();
      let img = clipboard.get_image();
      if img.is_err() {
        Err::<std::io::Error, &str>("Clipboard does not contain an image".into());
      } else {
        let im = img.unwrap();
        println!("Clipboard image w: {}", im.width.to_string());
        println!("Clipboard image h: {}", im.height.to_string());

        let mut stroo = "".to_string();
        for x in 0..im.bytes.len() {
          let v = im.bytes[x];
          stroo.push(v as char);
      }

        app.emit_all("clipboard_img", Payload { message: stroo }).unwrap();
        // println!("Clipboard image b: {}", stroo);
        return (); 
      }
      // println!("Clipboard image: {}", img.width.to_string());
      
      save_clipboard(clipboard.get_text().unwrap().to_string(), true, &app);
    }
  });

  CKey.bind(move || {
    if LControlKey.is_pressed() {
      // before sleep we get access to prev clipboard data
      sleep(Duration::from_millis(10));

      // here we have just recently clipboard data
      let mut clipboard = Clipboard::new().unwrap();
      println!("Clipboard text: {}", clipboard.get_text().unwrap());
      save_clipboard(clipboard.get_text().unwrap().to_string(), true, &app_clone);
    }
  });


  inputbot::handle_input_events();

  return Ok(123.to_string());
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      enable_clipboard, 
      remove_clipboard_item, 
      move_clipboard_item
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}