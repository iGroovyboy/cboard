use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::{thread, time};
use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;
use rdev::{Event, EventType, Key as inKey, listen, Keyboard as rdevKeyboard, KeyboardState};
use enigo::{Enigo, Settings, Key as outKey, Keyboard};
use enigo::Direction::{Press, Release};
use crate::filesys::{FILENAME_AUTO_REPLACEMENT};
use crate::helpers::{get_tauri_handle};
use crate::keyboard_layouts::get_current_keyboard_layout;
use parking_lot::Mutex;

#[derive(Debug)]
pub struct KeyEvent {
    event: Event,
    locale: String,
}

#[derive(Debug)]
pub struct KeyLog {
    keys: Vec<KeyEvent>,
}

impl Default for KeyLog {
    fn default() -> Self {
        Self {
            keys: Vec::new()
        }
    }
}

#[derive(Deserialize, Debug)]
struct KeyValue {
    key: String,
    value: String,
}

/// Holds only letters
pub static KEY_LOG: OnceLock<Arc<Mutex<KeyLog>>> = OnceLock::new();

fn initialize_key_log(log: &'static OnceLock<Arc<Mutex<KeyLog>>>) {
    log.set(Arc::new(Mutex::new(KeyLog::default()))).unwrap();
}

fn clear_key_log() {
    let mut auto_repl_buf = KEY_LOG.get().expect("KEY_LOG_AUTO_REPLACEMENT must have starting value").lock();
    auto_repl_buf.keys = Vec::new();
}

fn auto_repl_buffer_string() -> Option<String> {
    let key_log = KEY_LOG.get().expect("KEY_LOG_AUTO_REPLACEMENT must have starting value").lock();
    if key_log.keys.is_empty() {
        return None;
    }

    Some(
        key_log.keys.iter()
            .map(|e| e.event.name.clone().unwrap())
            .collect::<Vec<String>>()
            .join("")
    )
}

pub type UserAutoReplMap = HashMap<String, String>;

pub static USER_MAP: OnceLock<Arc<Mutex<UserAutoReplMap>>> = OnceLock::new();

fn initialize_auto_repl_map() {
    let mut map: UserAutoReplMap = HashMap::new();
    map.insert("rrr".to_string(), "replacement for rrr ❤️ ыы!".to_string());

    USER_MAP.set(Arc::new(Mutex::new(map))).unwrap();
}

fn is_user_auto_repl_map_empty() -> bool {
    USER_MAP.get().unwrap().lock().is_empty()
}

fn set_auto_replacement_data(new_data: Vec<KeyValue>) {
    let mut map = USER_MAP.get().unwrap().lock();

    map.clear();

    for item in new_data {
        map.insert(item.key, item.value);
    }
}

/// Used to block key logging when we send keys
pub static IS_SENDING: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

/// Used to block key logging when we send keys
fn is_sending() -> bool {
    if let Some(is_sending) = IS_SENDING.get() {
        let is_sending = is_sending.lock();
        *is_sending
    } else {
        false
    }
}

/// Used to block key logging when we send keys
fn set_is_sending(value: bool) {
    if let Some(is_sending) = IS_SENDING.get() {
        let mut is_sending = is_sending.lock();
        *is_sending = value;
    }
}

#[allow(dead_code)]
#[tauri::command]
pub fn update_auto_replace_data() -> Result<(), String> {
    let app = get_tauri_handle().clone();
    let from = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join("data")
        .join(FILENAME_AUTO_REPLACEMENT);

    let file = File::open(from).unwrap();
    let reader = BufReader::new(file);

    let data: Vec<KeyValue> = serde_json::from_reader(reader).expect("Failed to parse auto replacement JSON");
    set_auto_replacement_data(data);

    Ok(())
}

pub fn enable_key_listener() {
    initialize_auto_repl_map();
    initialize_key_log(&KEY_LOG);
    IS_SENDING.set(Arc::new(Mutex::new(false))).unwrap();
    update_auto_replace_data().unwrap();

    thread::spawn(move || {
        listen(move |event| {
            handle_event(event);
        }).unwrap();
    });
}

fn handle_event(event: Event) {
    if is_sending() || is_user_auto_repl_map_empty() {
        return;
    }

    if let EventType::KeyPress(key) = event.event_type {
        save_auto_replacement_log(&key, event.clone());
    }
}

fn save_auto_replacement_log(key: &inKey, event: Event) {
    let key_str = format!("{:?}", key);

    let extra_keys = [inKey::LeftBracket, inKey::RightBracket, inKey::SemiColon,
        inKey::Quote, inKey::Comma, inKey::Dot, inKey::BackSlash, inKey::Slash, inKey::Space];

    let num_row = [inKey::BackQuote, inKey::Num1, inKey::Num2, inKey::Num3,
        inKey::Num4, inKey::Num5, inKey::Num6, inKey::Num7, inKey::Num8, inKey::Num9,
        inKey::Num0, inKey::Minus, inKey::Equal];

    // here may be some edge cases when modifiers reset buffer when shouldn't
    if !key_str.starts_with("Key") && !extra_keys.contains(key) && !num_row.contains(key) {
        clear_key_log();
        return;
    }

    if let Some(name) = &event.name {
        if !is_valid_key_name(name) {
            return
        }
    } else {
        return
    }

    KEY_LOG.get().expect("KEY_LOG_AUTO_REPLACEMENT must have starting value").lock()
        .keys.push(KeyEvent {
            locale: get_current_keyboard_layout().unwrap_or_else(|| String::from("")),
            event,
        });

    println!("====BUF> {:#?}", auto_repl_buffer_string().unwrap());
    handle_auto_replacement();
}

fn is_valid_key_name(name: &String) -> bool {
    !contains_escape_string(name) || name.chars().all(|c|
        c.is_alphanumeric() || c.is_ascii_punctuation() || c.is_whitespace()
    )
}

fn contains_escape_string(s: &String) -> bool {
    s.chars().any(|c| c.is_ascii_control())
}

/// Handles the automatic replacement of text in the buffer.
///
/// If the buffer contains any key that should be replaced according to the
/// `USER_AUTO_REPLACEMENT_MAP`, the function replaces the key with its
/// corresponding value.
fn handle_auto_replacement() {
    match auto_repl_buffer_string() {
        Some(buf) => {
            let user_auto_repl_map = USER_MAP.get().unwrap().lock();
            if !user_auto_repl_map.keys().any(|k| buf.contains(k)) {
                return;
            }

            let map_key = user_auto_repl_map.keys().find(|&kk| buf.contains(&*kk)).unwrap().clone();
            let replacement = user_auto_repl_map.get(&map_key).unwrap().clone();

            clear_key_log();

            set_is_sending(true);

            // without thread this will perform actions BEFORE last symbols is typed in a window
            thread::spawn(move || {
                // remove n chars
                send_key_times(outKey::Backspace, map_key.chars().count() as i32).unwrap();

                send_string(&replacement).unwrap();
            });

            set_is_sending(false);
        },
        None => {
            return;
        },
    }

}

// TODO: move to separate mod
fn send_delayed_keypress(key: outKey, delay_ms: Option<u64>) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    enigo.key(key, Press).unwrap();
    if let Some(t) = delay_ms { thread::sleep(time::Duration::from_millis(t)) }

    enigo.key(key, Release).unwrap();
    if let Some(t) = delay_ms { thread::sleep(time::Duration::from_millis(t)) }

    Ok(())
}

// TODO: move to separate mod
// May send rdev::Unknown struct
fn send_string(string: &str) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.text(string).unwrap();

    Ok(())
}

// TODO: move to separate mod
fn send_key_times(key: outKey, len: i32) -> Result<(), String> {
    for _ in 0..len {
        send_delayed_keypress(key, None)?;
    }

    Ok(())
}