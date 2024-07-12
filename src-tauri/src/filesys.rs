use std::fs;

use tauri::Manager;

#[derive(Clone, serde::Serialize)]
pub struct Payload {
    pub message: String,
}

pub fn remove_extra_files(folder: String, max_files_count: i32, app: &tauri::AppHandle) {
    let path = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join("data")
        .join(folder);

    let filesCount = (fs::read_dir(&path).unwrap().count() as i32) + 1;
    println!("Count: {}", &filesCount);

    let mut files = fs::read_dir(&path).unwrap();
    if filesCount > max_files_count {
        let mut leftToRemove = filesCount - max_files_count;

        while let Some(file) = files.next() {
            if leftToRemove < 1 {
                break;
            }
            leftToRemove -= 1;

            fs::remove_file(file.unwrap().path()).unwrap();
        }
    }
}

#[tauri::command]
pub fn deleteAllByFolder(folder: String, app: tauri::AppHandle) {
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
    app.emit_all("clipboard", Payload { message: "remove_clipboard_items".to_string() }).unwrap();
}

#[tauri::command]
pub fn remove_clipboard_item(filename: String, folder: String, app: tauri::AppHandle) { // TODO: use ClipboardItem
    let file = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join("data")
        .join(folder)
        .join(filename);

    fs::remove_file(&file);
    println!("removed file {:?}", file);
    app.emit_all("clipboard", Payload { message: "remove_clipboard_item".to_string() }).unwrap();
}


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

    fs::rename(from, &to);
    println!("moved file {} to {:?}", &filename, to);
    app.emit_all("clipboard", Payload { message: "move_clipboard_item".to_string() }).unwrap();
}
