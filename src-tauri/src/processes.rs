use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, Process, ProcessStatus, RefreshKind, System};
use winapi::shared::minwindef::{BOOL, LPARAM};
use winapi::shared::windef::HWND;
use winapi::um::winuser::{EnumWindows, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible};
use tokio::time::{sleep, Duration};
use crate::filesys::{read_json_data, FILENAME_APPS_BLACKLIST};
use parking_lot::Mutex;

static IS_APP_ACTIVE: AtomicBool = AtomicBool::new(true);

pub fn app_active_state() -> bool {
    IS_APP_ACTIVE.load(Ordering::Relaxed)
}

pub fn set_app_active_state(state: bool) {
    IS_APP_ACTIVE.store(state, Ordering::Relaxed); // 200 ns
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyProcess {
    pid: u32,
    title: String,
    filename: String,
    filepath: String,
}

#[derive(Debug, Deserialize)]
pub struct BlacklistItem {
    filepath: String,
    enabled: bool,
    filename: Option<String>,
    title: Option<String>,
}

struct SystemProcesses {
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
        self.sys.refresh_processes(sysinfo::ProcessesToUpdate::All);  // 20ms
        let processes = self.sys.processes();

        let mut list: Vec<MyProcess> = Vec::new();
        for (pid, title) in foreground_apps().iter() {
            let p = processes.get(&Pid::from_u32(*pid)).unwrap();
            list.push(MyProcess{
                pid: *pid,
                title: title.clone(),
                filename: p.name().to_string_lossy().to_string(),
                filepath: p.exe().unwrap_or(Path::new("")).to_owned().to_string_lossy().to_string(),
            });
        }

        list
    }
}

pub async unsafe fn watch_active_window() {
    update_blacklist_data();
    let mut system_processes = SystemProcesses::new();

    loop {
        sleep(Duration::from_millis(500)).await;
        if is_blacklist_empty() {
            continue;
        }

        // let current_process = active_window(&mut system_processes);
        match active_window(&mut system_processes) {
            Some(current_process) => {
                let blacklist = get_blacklist_instance(); // 2µs 
                let mut blacklist = blacklist.lock();
        
                if let Some(blacklisted) = blacklist
                    .iter()
                    .find(|bl| bl.filepath == current_process.filepath ) {
                    if blacklisted.enabled {
                        println!("Blacklisted app: {:?}", blacklisted.filepath);
                        set_app_active_state(false);
                    }
                } else {
                    set_app_active_state(true);
                }
            },
            None => {},
        }
    }
}

pub static BLACKILST: OnceLock<Arc<Mutex<Vec<BlacklistItem>>>> = OnceLock::new();

fn is_blacklist_empty() -> bool {
    let blacklist = get_blacklist_instance();
    let mut blacklist = blacklist.lock();
    blacklist.is_empty()
}

fn get_blacklist_instance() -> Arc<parking_lot::Mutex<Vec<BlacklistItem>>> {
    BLACKILST.get_or_init(|| {
        Arc::new(parking_lot::Mutex::new(vec![]))
    }).clone()
}

fn set_blacklist_data(new_data: Vec<BlacklistItem>) {
    let blacklist = get_blacklist_instance();
    let mut blacklist = blacklist.lock();
    *blacklist = new_data;
}

#[allow(dead_code)]
#[tauri::command]
pub fn update_blacklist_data() -> Result<(), String> {
    match read_json_data::<Vec<BlacklistItem>>(FILENAME_APPS_BLACKLIST) {
        Ok(data) => {
            set_blacklist_data(data);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to read JSON: {}", FILENAME_APPS_BLACKLIST);
            Ok(())
        }
    }
}

pub unsafe fn active_window(system_processes: &mut SystemProcesses) -> Option<MyProcess> {
    let hwnd = GetForegroundWindow();

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);

    if let Some(process) = system_processes.processes().iter().find(|item| item.pid == pid) {
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
        list.push(MyProcess{
            pid: *pid,
            title: title.clone(),
            filename: p.name().to_string_lossy().to_string(),
            filepath: p.exe().unwrap_or(Path::new("")).to_owned().to_string_lossy().to_string(),
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
        EnumWindows(Some(enum_windows_proc), &mut foreground_apps as *mut _ as isize);
    }

    foreground_apps
}