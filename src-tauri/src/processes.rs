use crate::filesys::{read_json_data, write_json_data, FILENAME_APPS_BLACKLIST};
use core::time;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use sysinfo::{Pid, RefreshKind, System};
use winapi::shared::minwindef::{BOOL, LPARAM};
use winapi::shared::windef::{HWND, RECT};
use winapi::um::winuser::{EnumWindows, GetAncestor, GetDesktopWindow, GetForegroundWindow, GetShellWindow, GetSystemMetrics, GetWindowRect, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, GA_ROOTOWNER, SM_CXSCREEN, SM_CYSCREEN};
use crate::keyboard_layouts::{change_keyboard_layout, get_key_apps_instance, is_key_apps_empty, update_keyboard_layouts_data};

// TODO: refactor to use interface-like implementation to have same pub funcs for other os

/// Used throughout the app to check if app is currently enabled
static IS_APP_ACTIVE: AtomicBool = AtomicBool::new(true);

pub fn app_active_state() -> bool {
    IS_APP_ACTIVE.load(Ordering::Relaxed)
}

pub fn set_app_active_state(state: bool) {
    IS_APP_ACTIVE.store(state, Ordering::Relaxed); // 200 ns
}

/// Used throughout the app to check if app is fullscreen
static IS_FULLSCREEN: AtomicBool = AtomicBool::new(true);

pub fn is_fg_window_fullscreen() -> bool {
    IS_FULLSCREEN.load(Ordering::Relaxed)
}

pub fn set_is_fg_window_fullscreen(state: bool) {
    IS_FULLSCREEN.store(state, Ordering::Relaxed);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyProcess {
    pid: u32,
    title: String,
    filename: String,
    filepath: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlacklistItem {
    pub filepath: String,
    pub enabled: bool,
    pub filename: Option<String>,
    pub title: Option<String>,
}

pub struct SystemProcesses {
    sys: System,
}

impl SystemProcesses {
    fn new() -> Self {
        SystemProcesses {
            sys: sysinfo::System::new_with_specifics(RefreshKind::everything()), // 40ms
        }
    }

    fn processes(&mut self) -> Vec<MyProcess> {
        // Unless ProcessesToUpdate::All is used, dead processes are not removed from the set of processes kept in System
        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All); // 20ms
        let processes = self.sys.processes();

        let mut list: Vec<MyProcess> = Vec::new();
        for (pid, title) in foreground_apps().iter() {
            let p = processes.get(&Pid::from_u32(*pid)).unwrap();
            list.push(MyProcess {
                pid: *pid,
                title: title.clone(),
                filename: p.name().to_string_lossy().to_string(),
                filepath: p
                    .exe()
                    .unwrap_or(Path::new(""))
                    .to_owned()
                    .to_string_lossy()
                    .to_string(),
            });
        }

        list
    }
}

pub unsafe fn watch_active_window() {
    let _ = update_blacklist_data();
    let _ = update_keyboard_layouts_data();
    let mut system_processes = SystemProcesses::new();

    loop {
        thread::sleep(time::Duration::from_millis(500));

        let hwnd: HWND = GetForegroundWindow();
        if hwnd.is_null() {
            continue;
        }

        let hwnd = GetAncestor(hwnd, GA_ROOTOWNER);
        if hwnd.is_null() {
            continue;
        }

        if let Some(current_process) = active_window(&mut system_processes, Some(hwnd)) {
            // println!("------ {:#?}", current_process);

            handle_blacklist_window(&current_process);

            handle_full_screen_app(hwnd, &current_process);

            handle_keyboard_layout(hwnd, &current_process);
        }
    }
}

unsafe fn handle_keyboard_layout(hwnd: HWND, current_process: &MyProcess) {
    if is_key_apps_empty() {
        return;
    }

    let key_apps = get_key_apps_instance();
    let key_apps = key_apps.lock();

    if let Some(app) = key_apps
        .iter()
        .find(|a| a.filepath == current_process.filepath)
    {
        if app.enabled {
            change_keyboard_layout(hwnd, app.lang_id);
        } else {
            // TODO: change to previously used lang?
        }
    }
}

unsafe fn handle_blacklist_window(current_process: &MyProcess) {
    if is_blacklist_empty() {
        return;
    }

    let blacklist = get_blacklist_instance(); // 2µs
    let blacklist = blacklist.lock();

    if let Some(blacklisted) = blacklist
        .iter()
        .find(|bl| bl.filepath == current_process.filepath)
    {
        if blacklisted.enabled {
            println!("Blacklisted app: {:?}", blacklisted.filepath);
            set_app_active_state(false);
        }
    } else {
        set_app_active_state(true);
    }
}

unsafe fn handle_full_screen_app(hwnd: HWND, process: &MyProcess) {
    if !cfg!(target_os = "windows") {
        return;
    }

    if process.filename == "explorer.exe" {
        return;
    }

    match is_fullscreen(hwnd) {
        true => {
            println!("Fullscreen app found: {:?}", window_text(hwnd));
            set_is_fg_window_fullscreen(true);
        }
        _ => {
            set_is_fg_window_fullscreen(false);
        }
    };
}

pub static BLACKILST: OnceLock<Arc<Mutex<Vec<BlacklistItem>>>> = OnceLock::new();

fn is_blacklist_empty() -> bool {
    let blacklist = get_blacklist_instance();
    let blacklist = blacklist.lock();
    blacklist.is_empty()
}

fn get_blacklist_instance() -> Arc<parking_lot::Mutex<Vec<BlacklistItem>>> {
    BLACKILST
        .get_or_init(|| Arc::new(parking_lot::Mutex::new(vec![])))
        .clone()
}

fn set_blacklist_data(new_data: Option<Vec<BlacklistItem>>) -> Vec<BlacklistItem> {
    let blacklist = get_blacklist_instance();
    let mut blacklist = blacklist.lock();

    if let Some(data) = new_data {
        *blacklist = data.clone();
    }

    blacklist.clone()
}

#[allow(dead_code)]
#[tauri::command]
pub fn update_blacklist_data() -> Result<(), String> {
    match read_json_data::<Vec<BlacklistItem>>(FILENAME_APPS_BLACKLIST) {
        Ok(data) => {
            set_blacklist_data(Some(data));

            Ok(())
        }
        Err(_) => {
            let default_settings = set_blacklist_data(None);
            write_json_data(FILENAME_APPS_BLACKLIST, &default_settings);

            Ok(())
        }
    }
}

pub unsafe fn active_window(
    system_processes: &mut SystemProcesses,
    window: Option<HWND>,
) -> Option<MyProcess> {
    let hwnd = window.unwrap_or(GetForegroundWindow());

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);

    if let Some(process) = system_processes
        .processes()
        .iter()
        .find(|item| item.pid == pid)
    {
        return Some(process.clone());
    }

    None
}

pub fn processes() -> Vec<MyProcess> {
    let sys = sysinfo::System::new_with_specifics(RefreshKind::everything()); // 40ms
    let processes = sys.processes();

    let mut list: Vec<MyProcess> = Vec::new();
    for (pid, title) in foreground_apps().iter() {
        let p = processes.get(&Pid::from_u32(*pid)).unwrap();
        list.push(MyProcess {
            pid: *pid,
            title: title.clone(),
            filename: p.name().to_string_lossy().to_string(),
            filepath: p
                .exe()
                .unwrap_or(Path::new(""))
                .to_owned()
                .to_string_lossy()
                .to_string(),
        });
    }

    println!("{:#?}", list);
    list
}

#[tauri::command]
pub async fn get_proccesses_list() -> Result<String, String> {
    let data = processes();
    Ok(serde_json::to_string(&data).unwrap_or("oops".to_string()))
}

unsafe fn window_text(hwnd: HWND) -> String {
    let length = GetWindowTextLengthW(hwnd) + 1;
    let mut buffer: Vec<u16> = vec![0; length as usize];
    GetWindowTextW(hwnd, buffer.as_mut_ptr(), length);
    let window_title = String::from_utf16_lossy(&buffer[..length as usize - 1]);

    window_title
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let process_ids = &mut *(lparam as *mut HashSet<(u32, String)>);

    if IsWindowVisible(hwnd) == 0 {
        return 1;
    }

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid != 0 {
        process_ids.insert((pid, window_text(hwnd)));
    }

    1 // continue enumeration
}

fn foreground_apps() -> HashSet<(u32, String)> {
    let mut foreground_apps: HashSet<(u32, String)> = HashSet::new();
    unsafe {
        EnumWindows(
            Some(enum_windows_proc),
            &mut foreground_apps as *mut _ as isize,
        );
    }

    foreground_apps
}

#[cfg(target_os = "windows")]
pub unsafe fn is_fullscreen(hwnd: HWND) -> bool {
    let desktop_window = GetDesktopWindow();
    let shell_window = GetShellWindow();
    if hwnd.is_null() || hwnd == shell_window || hwnd == desktop_window {
        return false;
    }

    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    if GetWindowRect(hwnd, &mut rect) == 0 {
        return false;
    }

    let screen_width = GetSystemMetrics(SM_CXSCREEN);
    let screen_height = GetSystemMetrics(SM_CYSCREEN);

    rect.right - rect.left == screen_width && rect.bottom - rect.top == screen_height
}

#[cfg(not(target_os = "windows"))]
pub fn is_fullscreen() -> bool {
    false
}
