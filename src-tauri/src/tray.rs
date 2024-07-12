use tauri::{AppHandle, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, Manager};

pub fn make_tray() -> SystemTray {     // <- a function that creates the system tray
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
                hide_item_handle.set_title("Hide");
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
                    println!("bb");
                    std::process::exit(0);
                }
                "toggle" => {
                    let window = app.get_window("main").unwrap();
                    let hide_item_handle = app.tray_handle().get_item("toggle");

                    if window.is_visible().unwrap() {
                        window.hide().unwrap();
                        hide_item_handle.set_title("Show");
                    } else {
                        window.show().unwrap();
                        hide_item_handle.set_title("Hide");
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}