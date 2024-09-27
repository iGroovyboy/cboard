use crate::helpers::get_tauri_handle;
use tauri::Manager;

#[tauri::command]
pub fn hide_window() {
    let app = get_tauri_handle().clone();
    let window = app.get_window("main").unwrap();
    let menu_item = app.tray_handle().get_item("toggle");
    window.hide().unwrap();
    menu_item.set_title("Show").unwrap();
}

#[tauri::command]
pub fn show_window() {
    let app = get_tauri_handle().clone();
    let window = app.get_window("main").unwrap();
    let menu_item = app.tray_handle().get_item("toggle");
    window.show().unwrap();
    window.unminimize().unwrap();
    menu_item.set_title("Hide").unwrap();
}

#[tauri::command]
pub fn quit() {
    let app = get_tauri_handle().clone();
    let window = app.get_window("main").unwrap();
    window.close().unwrap();
}
