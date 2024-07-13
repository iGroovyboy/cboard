use tauri::{AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};

pub fn make_tray() -> SystemTray {
    let menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("toggle".to_string(), "Hide"))
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));

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
                "quit" => {
                    std::process::exit(0);
                }
                "toggle" => {
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