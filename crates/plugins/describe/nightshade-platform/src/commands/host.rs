#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn write_clipboard(text: &str) -> bool {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => clipboard.set_text(text.to_string()).is_ok(),
        Err(_) => false,
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn read_clipboard() -> Option<String> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    clipboard.get_text().ok()
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn open_url(url: &str) -> bool {
    webbrowser::open(url).is_ok()
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn write_clipboard(_text: &str) -> bool {
    false
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn read_clipboard() -> Option<String> {
    None
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub fn open_url(_url: &str) -> bool {
    false
}
