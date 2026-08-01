use crate::data::models::{KIT_TEXTURES, Kit, MODELS};

pub fn kit_textures(kit: Kit) -> &'static [(&'static str, &'static [u8])] {
    KIT_TEXTURES[kit as usize]
}

pub fn model(name: &str) -> usize {
    MODELS
        .iter()
        .position(|file| file.name == name)
        .unwrap_or_else(|| panic!("the kit has no model named {name}"))
}
