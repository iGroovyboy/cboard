#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use inputbot::{KeybdKey::*};

#[tauri::command]
fn my_custom_command() {
  println!("I was invoked from JS!");

  Numrow1Key.bind(|| println!("12345 days"));
  inputbot::handle_input_events();
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![my_custom_command])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}