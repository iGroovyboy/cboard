use rdev::{grab, Event, EventType, Key};
use crate::processes::{app_active_state, is_fg_window_fullscreen};

pub async fn watch_win_key() {
    // TODO: read config

    let callback = |event: Event| -> Option<Event> {
        if let EventType::KeyPress(Key::MetaLeft) = event.event_type {
            if app_active_state() && is_fg_window_fullscreen() {
                None
            } else {
                Some(event)
            }
        } 
        else { Some(event) }
    };

    if let Err(error) = grab(callback) {
        println!("Error catching win_key: {:?}", error)
    }

}
