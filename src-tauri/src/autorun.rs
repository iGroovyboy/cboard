use std::env;
use std::path::PathBuf;
use auto_launch::*;

pub fn autorun(value: bool) {
    match env::current_exe() {
        Ok(exe_path) => {
            let auto = AutoLaunchBuilder::new()
                .set_app_name("Cboard")
                .set_app_path(exe_path.to_str().unwrap())
                .set_use_launch_agent(true)
                .build()
                .unwrap();
    
            if value == true {
                let _ = auto.enable().is_ok();
            } else {
                let _ = auto.disable().is_ok();
            }
        },
        Err(e) => {
            eprintln!("Failed to get current executable path: {}", e);
        }
    }
}
