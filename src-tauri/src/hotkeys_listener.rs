use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use device_query::{DeviceQuery, DeviceState, Keycode};

use crate::{window};
use crate::settings::get_settings_instance;


#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Hotkeys {
    keys: Vec<Keycode>,
}

impl Hotkeys {
    pub fn new(keys: Vec<Keycode>) -> Self {
        Hotkeys { keys }
    }

    pub fn is_pressed(&self, keys: &Vec<Keycode>) -> bool {
        self.keys.iter().all(|k| keys.contains(k))
    }
}

type Callback<F> = Arc<Mutex<F>>;

pub struct HotkeysListener<F> 
where
    F: FnMut() + Send + 'static,
{
    subscribers: Arc<Mutex<HashMap<Hotkeys, Callback<F>>>>,
}

impl<F> HotkeysListener<F> 
where
    F: FnMut() + Send + 'static,
{
    pub fn new() -> Self {
        HotkeysListener {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn subscribe(&mut self, hotkey: Hotkeys, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        let mut subs = self.subscribers.lock().unwrap();
        subs.insert(hotkey, Arc::new(Mutex::new(callback)));
    }

    pub fn unsubscribe(&mut self, hotkey: Hotkeys) {
        let mut subs = self.subscribers.lock().unwrap();
        subs.remove(&hotkey);
    }

    pub fn listen(&self) {
        let subscribers = Arc::clone(&self.subscribers);
        
        thread::spawn(move || {
            let device_state = DeviceState::new();
            let mut prev_keys: Vec<Keycode> = vec![];

            loop {
                let keys: Vec<Keycode> = device_state.get_keys();

                if keys.is_empty() || keys == prev_keys  {
                    continue;
                }

                let subs = subscribers.lock().unwrap();
                for (hotkey, callback) in subs.iter() {
                    if hotkey.is_pressed(&keys) {
                        let mut cb = callback.lock().unwrap();
                        cb();
                    }

                }

                prev_keys = keys;
            }
        });
    }
}

// TODO: when user changes settings - must be unsubbed and restarted
pub fn run() {
    let mut hotkey_listener = HotkeysListener::new();

    let settings = get_settings_instance();
    let settings = settings.lock();

    println!(">>> {:#?}", settings.clone());

    match parse_keycodes(settings.show_app_hotkey.clone()) {
        Ok(hotkeys) => {
            hotkey_listener.subscribe(
                Hotkeys::new(hotkeys),
                move || {
                    window::show_window();
                },
            );
        },
        Err(err) => println!("Error parsing hotkeys: {}", err),
    }

    hotkey_listener.listen();
}

pub fn parse_keycodes(input: String) -> Result<Vec<Keycode>, String> {
    input
        .split(',')
        .map(|s| s.parse::<Keycode>())
        .collect::<Result<Vec<Keycode>, String>>()
}