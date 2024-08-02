use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use sysinfo::{Pid, Process, ProcessStatus, RefreshKind};
use winapi::shared::minwindef::{BOOL, LPARAM};
use winapi::shared::windef::HWND;
use winapi::um::winuser::{EnumWindows, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
pub struct MyProcess {
    pid: u32,
    title: String,
    filename: String,
    filepath: String,
}

pub async unsafe fn watch_active_window() {
    loop {
        sleep(Duration::from_millis(500)).await;
        println!("GO! {:#?}", active_window());
    }
}

pub unsafe fn active_window() -> (u32, String) {
    let hwnd = GetForegroundWindow();

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);

    (pid, window_text(hwnd))
}

pub fn processes() -> Vec<MyProcess> {
    let mut sys = sysinfo::System::new_with_specifics(RefreshKind::everything());
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