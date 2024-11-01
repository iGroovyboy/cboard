use crate::common::KeyValue;
use crate::filesys::{read_json_data, write_json_data, FILENAME_AUTO_REPLACEMENT};
use crate::keyboard_layouts::get_current_keyboard_layout_locale;
use crate::processes::app_active_state;
use parking_lot::Mutex;
use rdev::{listen, Event, EventType, Key as inKey};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::thread;
use crate::keys::{send_key_times, send_string};

#[allow(dead_code)]
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
        Self { keys: Vec::new() }
    }
}

/// Holds only letters
pub static KEY_LOG: OnceLock<Arc<Mutex<KeyLog>>> = OnceLock::new();

fn initialize_key_log(log: &'static OnceLock<Arc<Mutex<KeyLog>>>) {
    log.set(Arc::new(Mutex::new(KeyLog::default()))).unwrap();
}

fn clear_key_log() {
    let mut auto_repl_buf = KEY_LOG
        .get()
        .expect("KEY_LOG_AUTO_REPLACEMENT must have starting value")
        .lock();
    auto_repl_buf.keys = Vec::new();
}

fn auto_repl_buffer_string() -> Option<String> {
    let key_log = KEY_LOG
        .get()
        .expect("KEY_LOG_AUTO_REPLACEMENT must have starting value")
        .lock();
    if key_log.keys.is_empty() {
        return None;
    }

    Some(
        key_log
            .keys
            .iter()
            .map(|e| e.event.name.clone().unwrap())
            .collect::<Vec<String>>()
            .join(""),
    )
}

pub type UserAutoReplMap = HashMap<String, String>;

pub static USER_MAP: OnceLock<Arc<Mutex<UserAutoReplMap>>> = OnceLock::new();

fn initialize_auto_repl_map() {
    let mut map: UserAutoReplMap = HashMap::new();
    map.insert("<3".to_string(), "❤️".to_string());

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
    match read_json_data::<Vec<KeyValue>>(FILENAME_AUTO_REPLACEMENT) {
        Ok(data) => {
            set_auto_replacement_data(data);
        }
        Err(_) => {
            let default_settings = USER_MAP.get().unwrap().lock().clone();

            // must save this as array of objects with arbitrary strings
            let array: Vec<Value> = default_settings.into_iter()
                .map(|(key, value)| json!({ "key": key, "value": value }))
                .collect();
            let array = Value::Array(array);

            write_json_data(FILENAME_AUTO_REPLACEMENT, &array);
        }
    }

    Ok(())
}

pub fn enable_key_listener() {
    initialize_auto_repl_map();
    initialize_key_log(&KEY_LOG);
    IS_SENDING.set(Arc::new(Mutex::new(false))).unwrap();
    update_auto_replace_data().unwrap();

    let _ = thread::Builder::new()
        .name("auto_replacement:key_listener".to_string())
        .spawn(move || {
            listen(move |event| {
                if app_active_state() {
                    handle_event(event);
                }
            })
            .unwrap();
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

    let extra_keys = [
        inKey::LeftBracket,
        inKey::RightBracket,
        inKey::SemiColon,
        inKey::Quote,
        inKey::Comma,
        inKey::Dot,
        inKey::BackSlash,
        inKey::Slash,
        inKey::Space,
    ];

    let num_row = [
        inKey::BackQuote,
        inKey::Num1,
        inKey::Num2,
        inKey::Num3,
        inKey::Num4,
        inKey::Num5,
        inKey::Num6,
        inKey::Num7,
        inKey::Num8,
        inKey::Num9,
        inKey::Num0,
        inKey::Minus,
        inKey::Equal,
    ];

    // here may be some edge cases when modifiers reset buffer when shouldn't
    if !key_str.starts_with("Key") && !extra_keys.contains(key) && !num_row.contains(key) {
        clear_key_log();
        return;
    }

    if let Some(name) = &event.name {
        if !is_valid_key_name(name) {
            return;
        }
    } else {
        return;
    }

    KEY_LOG
        .get()
        .expect("KEY_LOG_AUTO_REPLACEMENT must have starting value")
        .lock()
        .keys
        .push(KeyEvent {
            locale: get_current_keyboard_layout_locale(),
            event,
        });

    handle_auto_replacement();
}

fn is_valid_key_name(name: &String) -> bool {
    !contains_escape_string(name)
        || name
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_ascii_punctuation() || c.is_whitespace())
}

fn contains_escape_string(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_control())
}

/// Handles the automatic replacement of text in the buffer.
///
/// If the buffer contains any key that should be replaced according to the
/// `USER_AUTO_REPLACEMENT_MAP`, the function replaces the key with its
/// corresponding value.
fn handle_auto_replacement() {
    if let Some(buf) = auto_repl_buffer_string() {
        let user_auto_repl_map = USER_MAP.get().unwrap().lock();
        if !user_auto_repl_map.keys().any(|k| buf.contains(k)) {
            return;
        }

        let map_key = user_auto_repl_map
            .keys()
            .find(|&kk| buf.contains(&*kk))
            .unwrap()
            .clone();
        let replacement = user_auto_repl_map.get(&map_key).unwrap().clone();

        clear_key_log();

        set_is_sending(true);

        // without thread this will perform actions BEFORE last symbols is typed in a window
        let _ = thread::Builder::new()
            .name("auto_replacement:send_keys".to_string())
            .spawn(move || {
                // remove n chars
                send_key_times(inKey::Backspace, map_key.chars().count() as i32).unwrap();

                send_string(&replacement).unwrap();
            });

        set_is_sending(false);
    }
}

