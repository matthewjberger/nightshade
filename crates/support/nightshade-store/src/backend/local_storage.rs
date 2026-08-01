pub fn write(folder: &str, name: &str, bytes: &[u8]) -> bool {
    write_text(folder, name, &to_text(bytes))
}

pub fn write_text(folder: &str, name: &str, text: &str) -> bool {
    let Some(storage) = storage() else {
        return false;
    };
    storage.set_item(&key(folder, name), text).is_ok()
}

pub fn read(folder: &str, name: &str) -> Option<Vec<u8>> {
    from_text(&read_text(folder, name)?)
}

pub fn read_text(folder: &str, name: &str) -> Option<String> {
    storage()?.get_item(&key(folder, name)).ok()?
}

pub fn remove(folder: &str, name: &str) -> bool {
    let Some(storage) = storage() else {
        return false;
    };
    storage.remove_item(&key(folder, name)).is_ok()
}

fn key(folder: &str, name: &str) -> String {
    format!("{folder}/{name}")
}

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn to_text(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(char::from_digit((byte >> 4) as u32, 16).unwrap_or('0'));
        text.push(char::from_digit((byte & 15) as u32, 16).unwrap_or('0'));
    }
    text
}

fn from_text(text: &str) -> Option<Vec<u8>> {
    let digits: Option<Vec<u8>> = text
        .chars()
        .map(|mark| mark.to_digit(16).map(|value| value as u8))
        .collect();
    let digits = digits?;
    if digits.len() % 2 != 0 {
        return None;
    }
    Some(
        digits
            .chunks(2)
            .map(|pair| (pair[0] << 4) | pair[1])
            .collect(),
    )
}
