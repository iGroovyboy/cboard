use core::time;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use device_query::{DeviceQuery, DeviceState, Keycode};
use parking_lot::{Mutex, Condvar};

use crate::{window};
use crate::settings::get_settings_instance;

static HOTKEYS_LISTENER: OnceLock<Arc<Mutex<HotkeysListener>>> = OnceLock::new();

pub fn get_hotkeys_listener_instance() -> Arc<parking_lot::Mutex<HotkeysListener>> {
    HOTKEYS_LISTENER.get_or_init(|| {
        Arc::new(parking_lot::Mutex::new(
            HotkeysListener::new()
        ))
            
    }).clone()
}

static GLOBAL_CONDVAR: OnceLock<Arc<(Mutex<bool>, Condvar)>> = OnceLock::new();

fn get_global_condvar() -> Arc<(Mutex<bool>, Condvar)> {
    GLOBAL_CONDVAR.get_or_init(|| Arc::new(
        (Mutex::new(false), Condvar::new())
    )).clone()
}

static IS_HOTKEYS_LISTENER: AtomicBool = AtomicBool::new(false);

pub fn hotkey_listener() -> bool {
    IS_HOTKEYS_LISTENER.load(Ordering::Relaxed)
}

pub fn set_hotkey_listener(state: bool) {
    IS_HOTKEYS_LISTENER.store(state, Ordering::Relaxed);
}

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

type Callback = Box<dyn Fn() + Send + Sync>;

pub struct HotkeysListener {
    subscribers: HashMap<Hotkeys, Callback>,
}

impl HotkeysListener {
    pub fn new() -> Self {
        HotkeysListener {
            subscribers: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, hotkey: Hotkeys, callback: Callback) {
        self.subscribers.insert(hotkey, callback);
    }

    pub fn unsubscribe(&mut self, hotkey: Hotkeys) {
        self.subscribers.remove(&hotkey);
    }

    pub fn clear(&mut self, hotkey: Hotkeys) {
        self.subscribers.clear();
    }
}

fn listen() {
    println!("listener started");

    thread::spawn(|| {
        let (lock, cvar) = &*get_global_condvar();

        let hotkeys_listener = get_hotkeys_listener_instance();
        let mut hotkeys_listener = hotkeys_listener.lock();

        let device_state = DeviceState::new();
        let mut prev_keys: Vec<Keycode> = vec![];

        set_hotkey_listener(true);

        loop {
            if !hotkey_listener() {
                println!("!hotkey_listener");
                let mut finished = lock.lock();
                *finished = true;
                cvar.notify_one();

                return;
            }

            let keys: Vec<Keycode> = device_state.get_keys();

            if keys.is_empty() || keys == prev_keys  {
                continue;
            }

            for (hotkey, callback) in hotkeys_listener.subscribers.iter() {
                if hotkey.is_pressed(&keys) {
                    callback();
                }

            }

            prev_keys = keys;
        }
    });
}


pub fn run() {
    // TODO: mb replace with tokio task?
    thread::spawn(|| {
        if hotkey_listener() {
            set_hotkey_listener(false);

            let (lock, cvar) = &*get_global_condvar();

            let mut finished = lock.lock();
            if !*finished {
                cvar.wait(&mut finished);
            }
            println!("Thread received signal.");
        }
        
        let hotkeys_listener = get_hotkeys_listener_instance();
        
        let mut hotkeys_listener = hotkeys_listener.lock();

        hotkeys_listener.subscribers.clear();

        let settings = get_settings_instance();
        let settings = settings.lock();

        println!(">>> {:#?}", settings.clone());
        match parse_keycodes(settings.show_app_hotkey.clone()) {
            Ok(hotkeys) => {
                hotkeys_listener.subscribe(
                    Hotkeys::new(hotkeys),
                    Box::new(|| {
                        window::show_window();
                    }),
                );
            },
            Err(err) => println!("Error parsing hotkeys: {}", err),
        }

    });

    listen();
}

pub fn parse_keycodes(input: String) -> Result<Vec<Keycode>, String> {
    input
        .split(',')
        .map(|s| s.parse::<Keycode>())
        .collect::<Result<Vec<Keycode>, String>>()
}