use tauri::{AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};

const STR_TOGGLE: &str = "toggle";
const STR_QUIT: &str = "quit";

pub fn make_tray() -> SystemTray {
    let menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new(STR_TOGGLE, "Hide"))
        .add_item(CustomMenuItem::new(STR_QUIT, "Quit"));

    return SystemTray::new().with_menu(menu);
}

pub fn handle_tray_events(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick {
            position: _,
            size: _,
            ..
        } => {
            let window = app.get_window("main").unwrap();
            let hide_item_handle = app.tray_handle().get_item("toggle");

            if !window.is_visible().unwrap() {
                window.show().unwrap();
                hide_item_handle.set_title("Hide").unwrap();
            }
        }
        SystemTrayEvent::RightClick {
            position: _,
            size: _,
            ..
        } => {
            println!("system tray received a right click");
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            match id.as_str() {
                STR_QUIT => {
                    //std::process::exit(0);
                    let window = app.get_window("main").unwrap();
                    window.close().unwrap();
                }
                STR_TOGGLE => {
                    let window = app.get_window("main").unwrap();
                    let hide_item_handle = app.tray_handle().get_item("toggle");

                    if window.is_visible().unwrap() {
                        window.hide().unwrap();
                        hide_item_handle.set_title("Show").unwrap();
                    } else {
                        window.show().unwrap();
                        hide_item_handle.set_title("Hide").unwrap();
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}