use crate::backend;

pub fn store_write(folder: &str, name: &str, bytes: &[u8]) -> bool {
    backend::write(folder, name, bytes)
}

pub fn store_write_text(folder: &str, name: &str, text: &str) -> bool {
    backend::write_text(folder, name, text)
}

pub fn store_remove(folder: &str, name: &str) -> bool {
    backend::remove(folder, name)
}
