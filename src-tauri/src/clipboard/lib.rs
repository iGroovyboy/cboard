use arboard::Clipboard;

pub fn has_image() -> bool {
    let mut clipboard = Clipboard::new().unwrap();
    match clipboard.get_image() {
        Ok(_) => true,
        Err(_) => false,
    }
}
