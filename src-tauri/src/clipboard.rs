use crate::filesys;
use crate::keys::send_paste_hotkeys;
use crate::processes::app_active_state;
use arboard::{Clipboard, Error, ImageData};
use parking_lot::Mutex;
use std::sync::{Arc, OnceLock};
use std::thread::sleep;
use std::time::Duration;
use std::{fs, thread};
use tauri::AppHandle;
use tokio::task;

// TODO: read from user settings
pub static MAX_CLIPBOARD_ITEMS: i32 = 150;

pub enum ClipboardContent<'a> {
    Text(String),
    Image(ImageData<'a>),
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct ClipboardItem {
    name: String,
    folder: String,
    path: String,
    contents: Option<String>,
}

pub struct FileTypes;

impl FileTypes {
    pub const TXT: &'static str = "txt";
    pub const PNG: &'static str = "png";
}

pub static CLIPBOARD: OnceLock<Arc<Mutex<Clipboard>>> = OnceLock::new();

pub static PREV_TEXT: OnceLock<Arc<Mutex<Option<String>>>> = OnceLock::new();

pub static PREV_IMAGE: OnceLock<Arc<Mutex<Option<ImageData>>>> = OnceLock::new();

pub mod my_clipboard {
    use std::fs;
    use std::sync::Arc;

    use arboard::Clipboard;
    use tauri::Manager;

    use crate::clipboard::{ClipboardContent, CLIPBOARD, MAX_CLIPBOARD_ITEMS};
    use crate::filesys;
    use crate::helpers;
    use crate::helpers::get_tauri_handle;

    pub fn get_instance() -> Arc<parking_lot::Mutex<Clipboard>> {
        CLIPBOARD
            .get_or_init(|| {
                Arc::new(parking_lot::Mutex::new(
                    Clipboard::new().expect("Failed to create global clipboard instance"),
                ))
            })
            .clone()
    }

    pub fn has_image() -> bool {
        let clipboard = get_instance();
        let mut clipboard = clipboard.lock();
        match clipboard.get_image() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn has_text() -> bool {
        let clipboard = get_instance();
        let mut clipboard = clipboard.lock();
        match clipboard.get_text() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn save_contents(contents: ClipboardContent) {
        let default_folder = "clipboard".to_string();
        let app = get_tauri_handle().clone();

        let app_dir = app
            .path_resolver()
            .app_local_data_dir()
            .expect("Failed to resolve app local dir");

        let p = app_dir.as_path().join("data").join(&default_folder);

        println!("save_clipboard: {}", p.display());

        fs::create_dir_all(&p).unwrap();

        match contents {
            ClipboardContent::Text(data) => {
                let f = p.join([helpers::get_timestamp(), ".txt".to_string()].concat());
                text::save(&f, &data);
            }
            ClipboardContent::Image(data) => {
                let f = p.join([helpers::get_timestamp(), ".png".to_string()].concat());
                image::save(&f, &data).unwrap();
            }
        }

        filesys::remove_extra_files(default_folder, MAX_CLIPBOARD_ITEMS, &app);

        app.emit_all(
            "clipboard",
            filesys::Payload {
                message: String::from("contents"),
            },
        )
        .unwrap();
    }

    pub mod text {
        use std::fs;
        use std::path::PathBuf;
        use std::sync::Arc;

        use crate::clipboard::my_clipboard::get_instance;
        use crate::clipboard::{my_clipboard, ClipboardContent, PREV_TEXT};

        pub fn get_previous() -> Arc<parking_lot::Mutex<Option<String>>> {
            PREV_TEXT
                .get_or_init(|| Arc::new(parking_lot::Mutex::new(None)))
                .clone()
        }

        pub fn get_previous_text() -> Result<Option<String>, String> {
            let prev_text = get_previous();
            let prev_text = prev_text.lock();
            Ok(prev_text.clone())
        }

        pub fn set_previous_text(text: String) -> Result<(), String> {
            let prev_text = get_previous();
            let mut prev_text = prev_text.lock();
            *prev_text = Some(text);
            Ok(())
        }

        pub fn get() -> Result<String, String> {
            let clipboard = get_instance();
            let mut clipboard = clipboard.lock();
            clipboard.get_text().map_err(|e| e.to_string())
        }

        pub fn set(text: String) -> Result<(), String> {
            let clipboard = get_instance();
            let mut clipboard = clipboard.lock();
            clipboard.set_text(text).map_err(|e| e.to_string())
        }

        pub fn save(path: &PathBuf, contents: &String) {
            fs::write(path, contents).expect("Unable to write file");
        }

        pub fn on_copy() {
            let clipboard = my_clipboard::get_instance();
            let mut clipboard_lock = clipboard.lock();

            match clipboard_lock.get_text() {
                Ok(text) => {
                    let previous_text = get_previous_text().unwrap();
                    match previous_text {
                        None => {
                            set_previous_text(text.clone()).unwrap();
                            my_clipboard::save_contents(ClipboardContent::Text(text));
                        }
                        Some(prev_text) => {
                            if text != prev_text {
                                my_clipboard::save_contents(ClipboardContent::Text(text));
                            }
                        }
                    }
                }
                Err(_) => {
                    my_clipboard::image::on_copy();
                }
            }
        }

        //EXAMPLE
        // pub fn spawn_thread_and_write(text: String) -> Result<(), String> {
        //     let clipboard = get_clipboard();
        //     thread::spawn(move || {
        //         let mut clipboard = clipboard.lock().unwrap();
        //         clipboard.set_text(text).unwrap();
        //     }).join().map_err(|e| format!("Thread error: {:?}", e))?;
        //     Ok(())
        // }
    }

    pub mod image {
        use std::path::PathBuf;
        use std::sync::Arc;

        use crate::clipboard::{my_clipboard, ClipboardContent, PREV_IMAGE};
        use arboard::ImageData;
        use image::{ImageBuffer, Rgba};
        // use crate::{ClipboardContent, my_clipboard, PREV_IMAGE};

        pub fn get_previous() -> Arc<parking_lot::Mutex<Option<ImageData<'static>>>> {
            PREV_IMAGE
                .get_or_init(|| Arc::new(parking_lot::Mutex::new(None)))
                .clone()
        }

        pub fn get_prev_image_data() -> Result<Option<ImageData<'static>>, String> {
            let prev_image = get_previous();
            let prev_image = prev_image.lock();
            Ok(prev_image.clone())
        }

        pub fn init_prev_image() -> Result<(), String> {
            let prev_image = get_previous();
            let mut prev_image = prev_image.lock();
            *prev_image = None;
            Ok(())
        }

        pub fn set_prev_image(image_data: ImageData<'static>) -> Result<(), String> {
            let prev_image = get_previous();
            let mut prev_image = prev_image.lock();
            *prev_image = Some(image_data);
            Ok(())
        }

        pub fn eq(image_data1: &ImageData, image_data2: &ImageData) -> bool {
            image_data1.width == image_data2.width
                && image_data1.height == image_data2.height
                && image_data1.bytes[..] == image_data2.bytes[..]
        }

        pub fn save(
            path: &PathBuf,
            image_data: &ImageData,
        ) -> Result<(), Box<dyn std::error::Error>> {
            let buffer: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
                image_data.width as u32,
                image_data.height as u32,
                &image_data.bytes[..],
            )
            .ok_or("Failed to create image buffer")?;

            buffer
                .save_with_format(path.as_path(), image::ImageFormat::Png)
                .unwrap();

            Ok(())
        }

        // EXAMPLE
        // pub fn spawn_thread_and_set_image(image_data: ImageData) -> Result<(), String> {
        //     let prev_image = get_prev_image();
        //     std::thread::spawn(move || {
        //         let mut prev_image = prev_image.lock().unwrap();
        //         *prev_image = Some(image_data);
        //     }).join().map_err(|e| format!("Thread error: {:?}", e))?;
        //     Ok(())
        // }

        pub fn on_copy() {
            let clipboard = my_clipboard::get_instance();
            let mut clipboard_lock = clipboard.lock();
            let image_data = clipboard_lock.get_image().unwrap();

            let previous_image = get_prev_image_data().unwrap();

            match previous_image {
                None => {
                    set_prev_image(image_data.clone()).expect("set_prev_image error");
                    my_clipboard::save_contents(ClipboardContent::Image(image_data));
                }
                Some(ref i) => {
                    if !eq(&i, &image_data) {
                        set_prev_image(image_data.clone()).expect("set_prev_image error");
                        my_clipboard::save_contents(ClipboardContent::Image(image_data));
                    }
                }
            }
        }
    }
}

pub static IMG_THREAD_INTERVAL: u64 = 1000;

pub fn enable_clipboard() -> Result<(), String> {
    filesys::create_folders(&[filesys::FOLDER_CLIPBOARD, filesys::FOLDER_FAVOURITES])
        .expect("Couldn't create required directories");

    // image can get to clipboard in many ways, so we use interval-based checker
    let _ = thread::Builder::new()
        .name("clipboard:image_checker".to_string())
        .spawn(move || {
            my_clipboard::image::init_prev_image().unwrap();

            loop {
                if !app_active_state() {
                    continue;
                }

                sleep(Duration::from_millis(IMG_THREAD_INTERVAL));

                if !my_clipboard::has_image() {
                    continue;
                }

                // TODO: will get clipboard and prev_image instances every cycle - mb profile it?
                my_clipboard::image::on_copy();
            }
        });

    Ok(())
}

#[tauri::command]
pub async fn paste(item: ClipboardItem, app: AppHandle) {
    // window::hide_window(app);
    // sleep(Duration::from_millis(50));
    let from = app
        .path_resolver()
        .app_local_data_dir()
        .expect("Failed to resolve app local dir")
        .as_path()
        .join("data")
        .join(&item.folder)
        .join(&item.name);

    task::spawn(async move {
        let mut clipboard = Clipboard::new().expect("Couldn't create Clipboard instance");
        match from.extension().unwrap().to_str().unwrap() {
            FileTypes::TXT => {
                let content = fs::read_to_string(from).unwrap();
                clipboard.set_html(&content, Some(&content)).unwrap();
            }
            FileTypes::PNG => {
                let img = image::io::Reader::open(from).unwrap().decode().unwrap();
                let rgba_image = img.to_rgba8();
                let (width, height) = rgba_image.dimensions();
                let bytes = rgba_image.into_raw();
                let image_data = arboard::ImageData {
                    width: width as usize,
                    height: height as usize,
                    bytes: std::borrow::Cow::Owned(bytes),
                };

                clipboard.set_image(image_data).unwrap();
            }
            &_ => {}
        }
    })
    .await
    .unwrap();

    send_paste_hotkeys();

    sleep(Duration::from_millis(50));
    clipboard_clear().unwrap();
}

fn clipboard_clear() -> Result<(), Error> {
    let mut clipboard = Clipboard::new().expect("Couldn't create Clipboard instance");
    clipboard.clear()?;

    Ok(())
}
