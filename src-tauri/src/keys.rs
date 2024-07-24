use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::{thread, time};
use enigo::{Enigo, Settings, Key as outKey, Keyboard};
use enigo::Direction::{Press, Release};
use rdev::{Event, EventType, Key as inKey, listen, Keyboard as rdevKeyboard, KeyboardState};
use crate::helpers::{is_alphabetic_or_space, print_type_of};
use crate::input_lang::get_current_keyboard_layout;

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

pub type UserAutoReplMap = HashMap<String, String>;

pub static USER_AUTO_REPLACEMENT_MAP: OnceLock<Arc<Mutex<UserAutoReplMap>>> = OnceLock::new();

/// Used to block key logging when we send keys
pub static IS_SENDING: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

/// Used to block key logging when we send keys
fn is_sending() -> bool {
    if let Some(is_sending) = IS_SENDING.get() {
        let is_sending = is_sending.lock().unwrap();
        *is_sending
    } else {
        false
    }
}

/// Used to block key logging when we send keys
fn set_is_sending(value: bool) {
    if let Some(is_sending) = IS_SENDING.get() {
        let mut is_sending = is_sending.lock().unwrap();
        *is_sending = value;
    }
}

pub fn initialize_auto_repl_map() {
    let mut map: UserAutoReplMap = HashMap::new();
    map.insert("rrr".to_string(), "replacement for rrr ❤️ ыы!".to_string());

    USER_AUTO_REPLACEMENT_MAP.set(Arc::new(Mutex::new(map))).unwrap();
}

/// Holds only letters
pub static KEY_LOG_AUTO_REPLACEMENT: OnceLock<Arc<Mutex<KeyLog>>> = OnceLock::new();

fn initialize_key_log(log: &'static OnceLock<Arc<Mutex<KeyLog>>>) {
    log.set(Arc::new(Mutex::new(KeyLog::default()))).unwrap();
}

pub fn enable_key_listener() {
    initialize_auto_repl_map();
    initialize_key_log(&KEY_LOG_AUTO_REPLACEMENT);
    IS_SENDING.set(Arc::new(Mutex::new(false))).unwrap();

    thread::spawn(move || {
        listen(move |event| {
            handle_event(event);
        }).unwrap();
    });
}

fn handle_event(event: Event) {
    if is_sending() {
        return;
    }

    if let EventType::KeyRelease(key) = event.event_type {
        // TODO: mb fix this hack to get name for KeyRelease, bc rdev listen doesn't save name for KeyRelease
        // rdev-0.5.3/src/windows/listen.rs:24
        let mut keyboard = rdevKeyboard::new().unwrap();
        let name = keyboard.add(&EventType::KeyPress(key));

        save_auto_replacement_log(&key, Event {
            name,
            ..event
        });
    }
}

// TODO: add check if auto_replacement enabled and has values - then ok
fn save_auto_replacement_log(key: &inKey, event: Event) {
    let key_str = format!("{:?}", key);

    // for eng use only main letters
    // for other langs use extra keys that may have letters
    let extra_letters = [inKey::LeftBracket, inKey::RightBracket, inKey::SemiColon, inKey::Quote, inKey::Comma, inKey::Dot];
    let current_keyboard_layout = get_current_keyboard_layout().unwrap_or_else(|| String::from(""));
    let is_english = current_keyboard_layout.starts_with("en");
    if (is_english || !extra_letters.contains(key)) && !key_str.starts_with("Key") {
        return;
    }

    if !is_valid_key_name(&event.name.clone().unwrap()) {
        return;
    }

    KEY_LOG_AUTO_REPLACEMENT.get().expect("KEY_LOG_AUTO_REPLACEMENT must have starting value").lock()
        .unwrap()
        .keys.push(KeyEvent {
            locale: get_current_keyboard_layout().unwrap_or_else(|| String::from("")),
            event,
        });

    println!("====BUF> {:#?}", get_auto_repl_buffer_string());
    handle_auto_replacement();

    // println!("LOG {:#?}", KEY_LOG_AUTO_REPLACEMENT.get().unwrap().lock().unwrap());
}

fn is_valid_key_name(name: &String) -> bool {
    !contains_escape_string(name) || is_alphabetic_or_space(name)
}

/// Handles the automatic replacement of text in the buffer.
///
/// If the buffer contains any key that should be replaced according to the
/// `USER_AUTO_REPLACEMENT_MAP`, the function replaces the key with its
/// corresponding value.
fn handle_auto_replacement() {
    match get_auto_repl_buffer_string() {
        Some(buf) => {
            let user_auto_repl_map = USER_AUTO_REPLACEMENT_MAP.get().expect("USER_AUTO_REPLACEMENT_MAP must have starting value").lock().unwrap();
            if !user_auto_repl_map.keys().any(|k| buf.contains(k)) {
                return;
            }

            let map_key = user_auto_repl_map.keys().find(|&kk| buf.contains(&*kk)).unwrap();
            let replacement = user_auto_repl_map.get(map_key).unwrap();

            // clear buf
            let mut auto_repl_buf = KEY_LOG_AUTO_REPLACEMENT.get().expect("KEY_LOG_AUTO_REPLACEMENT must have starting value").lock().unwrap();
            auto_repl_buf.keys = Vec::new();

            set_is_sending(true);
            // remove n chars
            send_key_times(outKey::Backspace, map_key.chars().count() as i32).unwrap();

            // type replacement
            send_string(replacement).unwrap();

            set_is_sending(false);
        },
        None => {
            return;
        },
    }

}

fn get_auto_repl_buffer_string() -> Option<String> {
    let x = KEY_LOG_AUTO_REPLACEMENT.get().expect("KEY_LOG_AUTO_REPLACEMENT must have starting value").lock().unwrap();
    if x.keys.is_empty() {
        return None;
    }

    Some(
        x.keys.iter()
        .map(|e| e.event.name.clone().unwrap())
        .collect::<Vec<String>>()
        .join("")
    )
}

fn contains_escape_string(s: &String) -> bool {
    s.chars().any(|c| c.is_ascii_control())
}

fn send_delayed_keypress(key: outKey, delay_ms: Option<u64>) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();

    enigo.key(key, Press).unwrap();
    if let Some(t) = delay_ms { thread::sleep(time::Duration::from_millis(t)) }

    enigo.key(key, Release).unwrap();
    if let Some(t) = delay_ms { thread::sleep(time::Duration::from_millis(t)) }

    Ok(())
}

// May send rdev::Unknown struct
fn send_string(string: &str) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.text(string).unwrap();

    Ok(())
}

fn send_key_times(key: outKey, len: i32) -> Result<(), String> {
    for _ in 0..len {
        send_delayed_keypress(outKey::Backspace, None)?;
    }

    Ok(())
}

fn convert_in_to_out_key(in_key: inKey) -> outKey {
    // TODO: convert_in_to_out_key
    outKey::Backspace
}


