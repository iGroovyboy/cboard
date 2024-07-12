use std::path::PathBuf;
use arboard::ImageData;
use image::{ImageBuffer, Rgba};


// pub fn save_image() {
    // let img = clipboard.get_image();
    // if img.is_err() {
    //   Err::<std::io::Error, &str>("Clipboard does not contain an image".into());
    // } else {
    //   let im = img.unwrap();
    //   println!("Clipboard image w: {}", im.width.to_string());
    //   println!("Clipboard image h: {}", im.height.to_string());
    //
    //   let mut stroo = "".to_string();
    //   for x in 0..im.bytes.len() {
    //     let v = im.bytes[x];
    //     stroo.push(v as char);
    //   }
    //
    //   app.emit_all("clipboard_img", Payload { message: stroo }).unwrap();
    //   // println!("Clipboard image b: {}", stroo);
    //   return ();
    // }
    // // println!("Clipboard image: {}", img.width.to_string());
    //
    // save_clipboard(clipboard.get_text().unwrap().to_string(), true, &app);
// }
