use crate::data::Save;
use crate::queries::{save_file, save_header_line};
use nightshade_store::prelude::{store_remove, store_write};

pub fn save_write<T: Save>(slot: u32, label: &str, value: &T) -> bool {
    if slot >= T::SLOTS {
        return false;
    }
    let Some(payload) = T::CODEC.encode(value) else {
        return false;
    };
    let mut bytes = save_header_line::<T>(label).into_bytes();
    bytes.extend_from_slice(&payload);
    store_write(T::FOLDER, &save_file::<T>(slot), &bytes)
}

pub fn save_remove<T: Save>(slot: u32) -> bool {
    store_remove(T::FOLDER, &save_file::<T>(slot))
}
