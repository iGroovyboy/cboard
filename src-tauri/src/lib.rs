mod helpers;
mod filesys;

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use arboard::{Clipboard, ImageData};
use image::{ImageBuffer, Rgba};
use tauri::Manager;
use crate::filesys::Payload;

// TODO: read from user settings
pub static MAX_CLIPBOARD_ITEMS: i32 = 150;

pub enum ClipboardContent<'a> {
    Text(String),
    Image(ImageData<'a>),
}

pub struct FileTypes;

impl FileTypes {
    pub const TXT: &'static str = "txt";
    pub const PNG: &'static str = "png";
}

// holds global clipboard instance
pub static CLIPBOARD: OnceLock<Arc<Mutex<Clipboard>>> = OnceLock::new();

pub static PREV_IMAGE: OnceLock<Arc<Mutex<Option<ImageData>>>> = OnceLock::new();

pub mod my_clipboard {
    use std::fs;
    use std::sync::{Arc, Mutex};
    use arboard::Clipboard;
    use tauri::Manager;
    use crate::{CLIPBOARD, ClipboardContent, filesys, helpers, MAX_CLIPBOARD_ITEMS};
    use crate::filesys::Payload;

    pub fn get_instance() -> Arc<Mutex<Clipboard>> {
        CLIPBOARD.get_or_init(|| {
            Arc::new(Mutex::new(Clipboard::new().expect("Failed to create global clipboard instance")))
        }).clone()
    }

    pub fn has_image() -> bool {
        let clipboard = get_instance();
        let mut clipboard = clipboard.lock().map_err(|e| e.to_string()).unwrap();
        match clipboard.get_image() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn has_text() -> bool {
        let clipboard = get_instance();
        let mut clipboard = clipboard.lock().map_err(|e| e.to_string()).unwrap();
        match clipboard.get_text() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn save_contents(contents: ClipboardContent, app: &tauri::AppHandle)
    {
        let default_folder = "clipboard".to_string();

        let app_dir = app
            .path_resolver()
            .app_local_data_dir()
            .expect("Failed to resolve app local dir");


        let p = app_dir.as_path()
            .join("data")
            .join(&default_folder);

        println!("save_clipboard: {}", p.display());

        fs::create_dir_all(&p).unwrap();

        match contents {
            ClipboardContent::Text(data) => {
                let f = p.join([helpers::get_timestamp(), ".txt".to_string()].concat());
                //fs::write(f, &contents).expect("Unable to write file");
                text::save(&f, &data);
            },
            ClipboardContent::Image(data) => {
                let f = p.join([helpers::get_timestamp(), ".png".to_string()].concat());
                image::save(&f, &data);
            }
        }

        filesys::remove_extra_files(default_folder, MAX_CLIPBOARD_ITEMS, &app);

        app.emit_all("clipboard", Payload { message: String::from("contents") }).unwrap();
    }

    pub mod text {
        use std::fs;
        use std::path::PathBuf;
        use crate::my_clipboard::get_instance;

        pub fn get() -> Result<String, String> {
            let clipboard = get_instance();
            let mut clipboard = clipboard.lock().map_err(|e| e.to_string())?;
            clipboard.get_text().map_err(|e| e.to_string())
        }

        pub fn set(text: String) -> Result<(), String> {
            let clipboard = get_instance();
            let mut clipboard = clipboard.lock().expect("Couldn't lock the clipboard instance");
            clipboard.set_text(text).map_err(|e| e.to_string())
        }

        pub fn save(path: &PathBuf, contents: &String) {
            fs::write(path, contents).expect("Unable to write file");
        }
        // fn read_from_clipboard() -> Result<String, String> {
        //     let clipboard = Self::get_clipboard();
        //     let mut clipboard = clipboard.lock().map_err(|e| e.to_string())?;
        //     clipboard.get_text().map_err(|e| e.to_string())
        // }

        // pub fn write_to_clipboard(text: String) -> Result<(), String> {
        //     let clipboard = Self::get_clipboard();
        //     let mut clipboard = clipboard.lock().expect("Couldn't lock the clipboard instance");
        //     clipboard.set_text(text).map_err(|e| e.to_string())
        // }

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
        use std::sync::{Arc, Mutex};
        use arboard::ImageData;
        use image::{ImageBuffer, Rgba};
        use crate::{my_clipboard, PREV_IMAGE};

        pub fn get_previous() -> Arc<Mutex<Option<ImageData<'static>>>> {
            PREV_IMAGE.get_or_init(|| {
                Arc::new(Mutex::new(None))
            }).clone()
        }

        pub fn get_prev_image_data() -> Result<Option<ImageData<'static>>, String> {
            let prev_image = get_previous();
            let prev_image = prev_image.lock().map_err(|e| e.to_string())?;
            Ok(prev_image.clone())
        }

        pub fn init_prev_image(image_data: ImageData<'static>) -> Result<(), String> {
            let prev_image = get_previous();
            let mut prev_image = prev_image.lock().map_err(|e| e.to_string())?;
            *prev_image = None;
            Ok(())
        }

        pub fn set_prev_image(image_data: ImageData<'static>) -> Result<(), String> {
            let prev_image = get_previous();
            let mut prev_image = prev_image.lock().map_err(|e| e.to_string())?;
            *prev_image = Some(image_data);
            Ok(())
        }

        pub fn eq(image_data1: &ImageData, image_data2: &ImageData) -> bool {
            image_data1.width == image_data2.width &&
                image_data1.height == image_data2.height &&
                image_data1.bytes[..] == image_data2.bytes[..]
        }

        pub fn save(path: &PathBuf, image_data: &ImageData) -> Result<(), Box<dyn std::error::Error>> {
            let buffer: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
                image_data.width as u32,
                image_data.height as u32,
                &image_data.bytes[..]
            ).ok_or("Failed to create image buffer")?;

            buffer.save_with_format(path.as_path(), image::ImageFormat::Png).unwrap();

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


        pub fn on_copy(previous_image: Option<ImageData>) {
            let clipboard = my_clipboard::get_instance();

            let mut clipboard_lock = clipboard.lock().unwrap();
            let image_data = clipboard_lock.get_image().unwrap();

            // match previous_image {
            //     None => {
            //         previous_image = Some(image_data.clone());
            //         save_clipboard(
            //             ClipboardContent::Image(image_data),
            //             &app_clone_img
            //         );
            //     },
            //     Some(ref i) => {
            //         if !image_eq(&previous_image.clone().unwrap(), &image_data) {
            //             previous_image = Some(image_data.clone());
            //             save_clipboard(
            //                 ClipboardContent::Image(image_data),
            //                 &app_clone_img
            //             );
            //         }
            //     },
            // }

            ()
        }

    }
}












