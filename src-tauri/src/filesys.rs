use std::fs;

use tauri::Manager;
use crate::APP_HANDLE;

pub const FOLDER_CLIPBOARD: &str = "clipboard";
pub const FOLDER_FAVOURITES: &str = "favorites";

#[derive(Clone, serde::Serialize)]
pub struct Payload {
    pub message: String,
}

#[allow(dead_code)]
pub fn remove_extra_files(folder: String, max_files_count: i32, app: &tauri::AppHandle) {
    let path = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join("data")
        .join(folder);

    let files_count = (fs::read_dir(&path).unwrap().count() as i32) + 1;
    println!("Count: {}", &files_count);

    let mut files = fs::read_dir(&path).unwrap();
    if files_count > max_files_count {
        let mut left_to_remove = files_count - max_files_count;

        while let Some(file) = files.next() {
            if left_to_remove < 1 {
                break;
            }
            left_to_remove -= 1;

            fs::remove_file(file.unwrap().path()).unwrap();
        }
    }
}

#[allow(dead_code)]
#[tauri::command]
pub fn delete_all_by_folder(folder: String, app: tauri::AppHandle) {
    let path = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join("data")
        .join(&folder);

    fs::remove_dir_all(&path).unwrap();
    fs::create_dir(&path).unwrap();
    println!("removed contents of {:?}", &folder);
    app.emit_all(
        "clipboard",
        Payload {
            message: "remove_clipboard_items".to_string(),
        },
    )
    .unwrap();
}

#[allow(dead_code)]
#[tauri::command]
pub fn remove_clipboard_item(filename: String, folder: String, app: tauri::AppHandle) {
    // TODO: use ClipboardItem
    let file = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join("data")
        .join(folder)
        .join(filename);

    fs::remove_file(&file).unwrap();
    println!("removed file {:?}", file);
    app.emit_all(
        "clipboard",
        Payload {
            message: "remove_clipboard_item".to_string(),
        },
    )
    .unwrap();
}

#[allow(dead_code)]
#[tauri::command]
pub fn move_clipboard_item(from: String, filename: String, folder: String, app: tauri::AppHandle) {
    let to = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join("data")
        .join(folder)
        .join(&filename);

    fs::rename(from, &to).unwrap();
    println!("moved file {} to {:?}", &filename, to);
    app.emit_all(
        "clipboard",
        Payload {
            message: "move_clipboard_item".to_string(),
        },
    )
    .unwrap();
}

#[allow(dead_code)]
pub fn create_folders<T: AsRef<str>>(folders: &[T]) -> std::io::Result<()> {
    // TODO: cannot import get_tauri_handle() directly - compiler can't find it in crate??????
    let app =  APP_HANDLE.get().expect("AppHandle is not set").clone();
    let to = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join("data");

    for folder in folders {
        let full_path = to.clone().join(folder.as_ref());
        fs::create_dir_all(full_path)?;
    }

    Ok(())
}
