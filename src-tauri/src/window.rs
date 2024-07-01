use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn hide_window(app: AppHandle) {
    let window = app.get_window("main").unwrap();
    let menu_item = app.tray_handle().get_item("toggle");
    window.hide();
    menu_item.set_title("Show");
}

#[tauri::command]
pub fn show_window(app: AppHandle) {
    let window = app.get_window("main").unwrap();
    let menu_item = app.tray_handle().get_item("toggle");
    window.show();
    window.unminimize();
    menu_item.set_title("Hide");
}

#[tauri::command]
pub fn quit() {
    std::process::exit(0);
}