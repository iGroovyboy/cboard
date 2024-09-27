use std::{thread, time};
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
        Err(SimulateError) => {
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
