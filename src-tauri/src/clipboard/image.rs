use std::path::PathBuf;
use arboard::ImageData;
use image::{ImageBuffer, Rgba};

pub fn image_eq(image_data1: &ImageData, image_data2: &ImageData) -> bool {
    image_data1.width == image_data2.width &&
    image_data1.height == image_data2.height &&
    image_data1.bytes[..] == image_data2.bytes[..]
}

pub fn save_to_file(path: &PathBuf, image_data: &ImageData) -> Result<(), Box<dyn std::error::Error>> {
    // Create an ImageBuffer from ImageData
    let buffer: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(
        image_data.width as u32,
        image_data.height as u32,
        &image_data.bytes[..]
    ).ok_or("Failed to create image buffer")?;

    // Save the ImageBuffer to a file
    buffer.save_with_format(path.as_path(), image::ImageFormat::Png).unwrap();

    Ok(())
}

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
