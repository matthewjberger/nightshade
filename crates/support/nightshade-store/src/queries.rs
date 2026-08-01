use crate::backend;

pub fn store_read(folder: &str, name: &str) -> Option<Vec<u8>> {
    backend::read(folder, name)
}

pub fn store_read_text(folder: &str, name: &str) -> Option<String> {
    backend::read_text(folder, name)
}
