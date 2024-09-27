use crate::clipboard::FileTypes;
use crate::helpers::get_tauri_handle;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::{
    fs::{self},
    io,
};
use tauri::Manager;
use tokio::task;

#[allow(dead_code)]
pub const FOLDER_DATA: &str = "data";
#[allow(dead_code)]
pub const FOLDER_CLIPBOARD: &str = "clipboard";
#[allow(dead_code)]
pub const FOLDER_FAVOURITES: &str = "favorites";
#[allow(dead_code)]
pub const FILENAME_AUTO_REPLACEMENT: &str = "autoreplace.json";
pub const FILENAME_APPS_BLACKLIST: &str = "blacklist.json";
pub const FILENAME_SETTINGS: &str = "settings.json";

pub const FILE_MAX_LENGTH: u8 = 255;

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
        .join(FOLDER_DATA)
        .join(folder);

    let files_count = fs::read_dir(&path).unwrap().count() as i32;
    let files = fs::read_dir(&path).unwrap();
    if files_count > max_files_count {
        let mut left_to_remove = files_count - max_files_count;

        for file in files {
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
pub async fn delete_all_by_folder(folder: String, app: tauri::AppHandle) {
    let path = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join(FOLDER_DATA)
        .join(&folder);

    if !path.is_dir() {
        println!("Not a directory: {:?}", &folder);
        return;
    }

    let mut tasks = Vec::new();

    if fs::read_dir(&path).is_err() {
        eprintln!("Couldn't read dir");
    }

    for entry in fs::read_dir(&path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            let task = task::spawn(async move {
                match fs::remove_file(&path) {
                    Ok(_) => println!("File removed: {:?}", path),
                    Err(e) => {
                        if e.kind() == io::ErrorKind::PermissionDenied {
                            eprintln!("File is locked or in use: {:?}", path);
                        } else {
                            eprintln!("Error removing file: {:?}", e);
                        }
                    }
                }
            });

            tasks.push(task);
        }
    }

    for task in tasks {
        let _ = task.await;
    }

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
        .join(FOLDER_DATA)
        .join(folder)
        .join(filename);

    if fs::remove_file(&file).is_ok() {
        println!("removed file {:?}", file);
        app.emit_all(
            "clipboard",
            Payload {
                message: "remove_clipboard_item".to_string(),
            },
        )
        .unwrap();
    } else {
        eprintln!("file doesn't exist {:?}", file);
    }
}

#[allow(dead_code)]
#[tauri::command]
pub async fn move_clipboard_item(
    from: String,
    filename: String,
    folder: String,
    app: tauri::AppHandle,
) {
    let to = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join(FOLDER_DATA)
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
    let app = get_tauri_handle().clone();
    let to = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join(FOLDER_DATA);

    for folder in folders {
        let full_path = to.clone().join(folder.as_ref());
        fs::create_dir_all(full_path)?;
    }

    Ok(())
}

trait PathBufTauri {
    fn asset_path(self) -> String;
}

impl PathBufTauri for PathBuf {
    fn asset_path(self) -> String {
        let url = utf8_percent_encode(self.to_str().unwrap(), NON_ALPHANUMERIC);

        // \tauri\core\tauri\scripts\core.js:12
        format!("https://asset.localhost/{url}")
    }
}

fn read_file_by_char_len(file_path: &PathBuf, max_len: u8) -> Result<String, io::Error> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::with_capacity(max_len as usize, file);
    let mut contents = String::new();

    let mut char_count: usize = 0;
    while char_count < max_len as usize {
        let mut single_char = [0; 1];
        let bytes_read = reader.read(&mut single_char).unwrap_or(0);

        if bytes_read == 0 {
            break; // End of file?
        }

        if let Ok(c) = std::str::from_utf8(&single_char) {
            contents.push_str(c);
            char_count += c.chars().count();
        }
    }

    Ok(contents)
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageFile {
    path: String,
    name: String,
    extension: String,
    folder: String,
    size: u64,
    contents: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageFolder {
    path: String,
    name: String,
    children: Vec<StorageFile>,
}

#[tauri::command]
pub async fn read_clipboard_data() -> Result<String, String> {
    let app = get_tauri_handle().clone();
    let dir = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join(FOLDER_DATA);

    if !dir.is_dir() {
        return Err("Specified path is not a dir".to_string());
    }

    let mut data: Vec<StorageFolder> = Vec::new();

    let entries = fs::read_dir(dir)
        .unwrap()
        .filter(|e| e.as_ref().unwrap().path().is_dir())
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();

    for subdir in entries {
        let files = fs::read_dir(&subdir)
            .unwrap()
            .collect::<Result<Vec<_>, io::Error>>()
            .unwrap();

        let mut children = Vec::new();

        for file in files {
            let extension = file
                .path()
                .extension()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let contents = match extension.as_str() {
                FileTypes::TXT => Some(
                    read_file_by_char_len(&file.path(), FILE_MAX_LENGTH).unwrap_or("".to_string()),
                ),
                FileTypes::PNG => Some(&file.path().asset_path()).cloned(),
                _ => None,
            };

            let file_data = StorageFile {
                contents,
                folder: subdir.file_name().unwrap().to_string_lossy().to_string(),
                name: file.file_name().to_string_lossy().to_string(),
                path: file.path().to_string_lossy().to_string(),
                extension,
                size: file.metadata().unwrap().len(),
            };

            children.push(file_data);
        }

        children.sort_by(|a, b| b.name.cmp(&a.name));

        data.push(StorageFolder {
            path: subdir.as_path().to_string_lossy().to_string(),
            name: subdir.file_name().unwrap().to_string_lossy().to_string(),
            children,
        });
    }

    Ok(serde_json::to_string(&data).unwrap_or("oops".to_string()))
}

pub fn read_json_data<T: DeserializeOwned>(
    filename: &str,
) -> Result<T, Box<dyn std::error::Error>> {
    let app = get_tauri_handle().clone();
    let from = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join("data")
        .join(filename);

    let file = File::open(from).unwrap();
    let reader = BufReader::new(file);

    let data: T = serde_json::from_reader(reader)?;

    Ok(data)
}
