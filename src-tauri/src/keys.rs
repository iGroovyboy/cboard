use std::{collections::HashMap, thread, time};
use device_query::Keycode;
use enigo::{Enigo, Keyboard, Settings};
use rdev::{EventType, Key};

// May send rdev::Unknown struct
pub fn send_string(string: &str) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    enigo.text(string).unwrap();

    Ok(())
}

fn send(event_type: &EventType) {
    match rdev::simulate(event_type) {
        Ok(()) => (),
        Err(_) => {
            println!("Couldn't send {:?}", event_type);
        }
    }
    thread::sleep(time::Duration::from_millis(20));
}

pub fn send_key_times(key: Key, len: i32) -> Result<(), String> {
    for _ in 0..len {
        send(&EventType::KeyPress(key));
        send(&EventType::KeyRelease(key));
    }

    Ok(())
}

// TODO: test/fix bind on linux/mac
pub fn send_paste_hotkeys() {
    send(&EventType::KeyPress(Key::ControlLeft));
    send(&EventType::KeyPress(Key::KeyV));
    send(&EventType::KeyRelease(Key::KeyV));
    send(&EventType::KeyRelease(Key::ControlLeft));
}

pub fn send_hotkeys(hotkeys: Option<Vec<Key>>) {
    if let None = hotkeys {
        return
    }

    if hotkeys.as_ref().unwrap().is_empty() {
        return;
    }

    for key in &hotkeys.clone().unwrap() {
        send(&EventType::KeyPress(*key));
    }

    for key in &hotkeys.unwrap() {
        send(&EventType::KeyRelease(*key));
    }
}

pub fn hotkeys_list_devicequery_to_rdev(hotkeys: Vec<device_query::Keycode>) -> Option<Vec<rdev::Key>> {
    if hotkeys.is_empty() {
        return None;
    }

    let key_mapping = key_mapping_devicequery_rdev();
    let mut rdev_keys = vec![];
    for key in hotkeys {
        if let Some(rdev_key) = key_mapping.get(&key) {
            rdev_keys.push(*rdev_key);
        }
    }

    Some(rdev_keys)
}

fn key_mapping_devicequery_rdev() -> HashMap<device_query::Keycode, rdev::Key> {
    let mut map = HashMap::new();
    
    map.insert(Keycode::Key0, Key::Num0);
    map.insert(Keycode::Key1, Key::Num1);
    map.insert(Keycode::Key2, Key::Num2);
    map.insert(Keycode::Key3, Key::Num3);
    map.insert(Keycode::Key4, Key::Num4);
    map.insert(Keycode::Key5, Key::Num5);
    map.insert(Keycode::Key6, Key::Num6);
    map.insert(Keycode::Key7, Key::Num7);
    map.insert(Keycode::Key8, Key::Num8);
    map.insert(Keycode::Key9, Key::Num9);
    map.insert(Keycode::A, Key::KeyA);
    map.insert(Keycode::B, Key::KeyB);
    map.insert(Keycode::C, Key::KeyC);
    map.insert(Keycode::D, Key::KeyD);
    map.insert(Keycode::E, Key::KeyE);
    map.insert(Keycode::F, Key::KeyF);
    map.insert(Keycode::G, Key::KeyG);
    map.insert(Keycode::H, Key::KeyH);
    map.insert(Keycode::I, Key::KeyI);
    map.insert(Keycode::J, Key::KeyJ);
    map.insert(Keycode::K, Key::KeyK);
    map.insert(Keycode::L, Key::KeyL);
    map.insert(Keycode::M, Key::KeyM);
    map.insert(Keycode::N, Key::KeyN);
    map.insert(Keycode::O, Key::KeyO);
    map.insert(Keycode::P, Key::KeyP);
    map.insert(Keycode::Q, Key::KeyQ);
    map.insert(Keycode::R, Key::KeyR);
    map.insert(Keycode::S, Key::KeyS);
    map.insert(Keycode::T, Key::KeyT);
    map.insert(Keycode::U, Key::KeyU);
    map.insert(Keycode::V, Key::KeyV);
    map.insert(Keycode::W, Key::KeyW);
    map.insert(Keycode::X, Key::KeyX);
    map.insert(Keycode::Y, Key::KeyY);
    map.insert(Keycode::Z, Key::KeyZ);
    map.insert(Keycode::F1, Key::F1);
    map.insert(Keycode::F2, Key::F2);
    map.insert(Keycode::F3, Key::F3);
    map.insert(Keycode::F4, Key::F4);
    map.insert(Keycode::F5, Key::F5);
    map.insert(Keycode::F6, Key::F6);
    map.insert(Keycode::F7, Key::F7);
    map.insert(Keycode::F8, Key::F8);
    map.insert(Keycode::F9, Key::F9);
    map.insert(Keycode::F10, Key::F10);
    map.insert(Keycode::F11, Key::F11);
    map.insert(Keycode::F12, Key::F12);

    map.insert(Keycode::Escape, Key::Escape);
    map.insert(Keycode::Space, Key::Space);
    map.insert(Keycode::LControl, Key::ControlLeft);
    map.insert(Keycode::RControl, Key::ControlRight);
    map.insert(Keycode::LShift, Key::ShiftLeft);
    map.insert(Keycode::RShift, Key::ShiftRight);
    
    // option key on macOS
    map.insert(Keycode::LAlt, Key::Alt);
    map.insert(Keycode::RAlt, Key::AltGr);
    map.insert(Keycode::Command, Key::MetaLeft);
    map.insert(Keycode::LOption, Key::Alt);
    map.insert(Keycode::ROption, Key::AltGr);
    map.insert(Keycode::LMeta, Key::MetaLeft);
    map.insert(Keycode::RMeta, Key::MetaRight);

    map.insert(Keycode::Enter, Key::Return);
    map.insert(Keycode::Backspace, Key::Backspace);
    map.insert(Keycode::CapsLock, Key::CapsLock);
    map.insert(Keycode::Tab, Key::Tab);

    map.insert(Keycode::Up, Key::UpArrow);
    map.insert(Keycode::Down, Key::DownArrow);
    map.insert(Keycode::Left, Key::LeftArrow);
    map.insert(Keycode::Right, Key::RightArrow);

    map.insert(Keycode::Home, Key::Home);
    map.insert(Keycode::End, Key::End);
    map.insert(Keycode::PageUp, Key::PageUp);
    map.insert(Keycode::PageDown, Key::PageDown);
    map.insert(Keycode::Insert, Key::Insert);
    map.insert(Keycode::Delete, Key::Delete);

    // Numpad keys which have not been implemented: NumpadSeparator NumLock
    map.insert(Keycode::Numpad0, Key::Kp0);
    map.insert(Keycode::Numpad1, Key::Kp1);
    map.insert(Keycode::Numpad2, Key::Kp2);
    map.insert(Keycode::Numpad3, Key::Kp3);
    map.insert(Keycode::Numpad4, Key::Kp4);
    map.insert(Keycode::Numpad5, Key::Kp5);
    map.insert(Keycode::Numpad6, Key::Kp6);
    map.insert(Keycode::Numpad7, Key::Kp7);
    map.insert(Keycode::Numpad8, Key::Kp8);
    map.insert(Keycode::Numpad9, Key::Kp9);
    map.insert(Keycode::NumpadSubtract, Key::KpMinus);
    map.insert(Keycode::NumpadAdd, Key::KpPlus);
    map.insert(Keycode::NumpadDivide, Key::KpDivide);
    map.insert(Keycode::NumpadMultiply, Key::KpMultiply);
    // Careful, on Windows KpReturn does not exist, it' s strictly equivalent to Return, also Keypad keys
    // get modified if NumLock is Off and ARE pagedown and so 
    map.insert(Keycode::NumpadEnter, Key::Return);

    // The following keys names represent the position of the key in a US keyboard,
    // not the sign value. In a different keyboards and OS, the position can vary.
    map.insert(Keycode::Grave, Key::BackQuote);
    map.insert(Keycode::Minus, Key::Minus);
    map.insert(Keycode::Equal, Key::Equal);
    map.insert(Keycode::LeftBracket, Key::LeftBracket);
    map.insert(Keycode::RightBracket, Key::RightBracket);
    map.insert(Keycode::BackSlash, Key::BackSlash);
    map.insert(Keycode::Semicolon, Key::SemiColon);
    map.insert(Keycode::Apostrophe, Key::Quote);
    map.insert(Keycode::Comma, Key::Comma);
    map.insert(Keycode::Dot, Key::Dot);
    map.insert(Keycode::Slash, Key::Slash);

    map
}