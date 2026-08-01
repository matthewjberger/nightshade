#[cfg(not(target_arch = "wasm32"))]
mod file;
#[cfg(target_arch = "wasm32")]
mod local_storage;

#[cfg(not(target_arch = "wasm32"))]
pub use file::{read, read_text, remove, write, write_text};
#[cfg(target_arch = "wasm32")]
pub use local_storage::{read, read_text, remove, write, write_text};
