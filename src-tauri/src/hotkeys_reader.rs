use device_query::{DeviceQuery, DeviceState, Keycode};
use std::{sync::atomic::{AtomicBool, Ordering}, thread, time};
use tauri::Manager;

use crate::helpers::get_tauri_handle;

static IS_HOTEKEYS_READER_ON: AtomicBool = AtomicBool::new(true);

pub fn hotkey_reader_on() -> bool {
    IS_HOTEKEYS_READER_ON.load(Ordering::Relaxed)
}

pub fn set_hotkey_reader_on(state: bool) {
    IS_HOTEKEYS_READER_ON.store(state, Ordering::Relaxed);
}

#[derive(Clone, serde::Serialize)]
struct Payload {
  keys: String,
}

#[tauri::command]
pub fn hotkeys_listen() {
    set_hotkey_reader_on(true);

    thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(hotkeys_reader());
    });
}

#[tauri::command]
pub fn hotkeys_unlisten() {
    set_hotkey_reader_on(false);
}

async fn hotkeys_reader() -> Vec<Keycode> {
    let app: tauri::AppHandle = get_tauri_handle().clone();

    let device_state = DeviceState::new();
    let mut prev_keys = vec![];

    loop {
        thread::sleep(time::Duration::from_millis(200));

        if !hotkey_reader_on() {
            return prev_keys;
        }

        let keys = device_state.get_keys();
        if keys != prev_keys && !keys.is_empty() {
            println!("[Keyboard] {:?}", keys);
            let _ = app.emit_all(
                "hotkeys_reader", 
                Payload { keys: format_keycodes(keys.clone())}
            );
        }
        
        prev_keys = keys;
    }
}

fn format_keycodes(keycodes: Vec<Keycode>) -> String {
    keycodes.iter()
            .map(|keycode| keycode.to_string())
            .collect::<Vec<String>>()
            .join(",")
}