#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod tray;
mod helpers;
mod filesys;
mod window;

use inputbot::{KeybdKey::*};
use arboard::Clipboard;
use std::{thread, thread::sleep, time::{Duration}, fs};
use tauri::{Manager, AppHandle};
use std::sync::{Arc, Mutex};
use tauri::State;

const MAX_CLIPBOARD_ITEMS: i32 = 150;

// the payload type must implement `Serialize` and `Clone`.
#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

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

// TODO: detect/save image
fn save_clipboard(contents: String, is_text: bool, app: &tauri::AppHandle) {
  let default_folder = "clipboard".to_string();

  let app_dir = app
    .path_resolver()
    .app_local_data_dir()
    .expect("Failed to resolve app local dir");


  let p = app_dir.as_path()
    .join("data")
    .join(&default_folder);
  
  let f =  p.join([helpers::get_timestamp(), ".txt".to_string()].concat());

  println!("save_clipboard: {}", p.display());

  fs::create_dir_all(p).unwrap();
  fs::write(f, &contents).expect("Unable to write file");

  filesys::remove_extra_files(default_folder, MAX_CLIPBOARD_ITEMS, &app);

  app.emit_all("clipboard", Payload { message: contents }).unwrap();
}



#[tauri::command]
fn enable_clipboard(app: tauri::AppHandle, state: State<ClipboardPreviousText>) -> Result<String, String> {
  let stateClone = Arc::clone(&state.text);
  let mut stateText= stateClone.lock().unwrap();
  *stateText = "wasd".to_string(); // TODO: load init here?

  println!("Clipboard management was enabled! {}", *stateText);

  let app_clone = app.clone();

  thread::spawn(move || {
    let mut i = 1;

    loop {
      if i > 1 {
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

  let stateClone = stateClone.clone();
  CKey.bind(move || {
    if LControlKey.is_pressed() {
      // before sleep we get access to prev clipboard data
      sleep(Duration::from_millis(10));

      // here we have just recently clipboard data
      let mut clipboard = Clipboard::new().unwrap();
      println!("Clipboard text: {}", clipboard.get_text().unwrap());

      let mut stateText = stateClone.lock().unwrap();
      if *stateText == clipboard.get_text().unwrap() {
        println!("Ignore: is dupe");
      } else {
        *stateText = clipboard.get_text().unwrap().to_string();
        save_clipboard(clipboard.get_text().unwrap().to_string(), true, &app_clone);
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

  let content = fs::read_to_string(from).unwrap();
  // let content2 = Box::leak(content.into_boxed_str());
  // KeySequence(content2).send();

  let mut clipboard = Clipboard::new().unwrap();
  clipboard.set_html(&content, Some(&content)).unwrap();

  LControlKey.press();
  VKey.press();
  VKey.release();
  LControlKey.release();

  sleep(Duration::from_millis(50));
  clipboard.clear();

  // KeySequence("ffff").send();

  // let mut enigo = Enigo::new();
  // enigo.mouse_move_to(500, 200);
  // enigo.key_down(Key::Control);
  // enigo.key_click(Key::Layout('v'));
  // enigo.key_up(Key::Control);

  // enigo.key_sequence_parse("{+SHIFT}Hello World{-SHIFT}");

  println!("END PASTE");
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