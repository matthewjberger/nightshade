use nightshade_save::data::{Save, SaveCodec};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct Kept {
    pub names: Vec<String>,
    pub tags: Vec<u8>,
    pub widths: Vec<u8>,
    pub parts: Vec<[f32; 4]>,
    pub lines: Vec<String>,
}

impl Save for Kept {
    const FOLDER: &'static str = "nightshade-description";
    const NAME: &'static str = "kept";
    const VERSION: u32 = 1;
    const CODEC: SaveCodec = SaveCodec::Json;
}

pub fn place(kept: &Kept, name: &str) -> Option<usize> {
    kept.names.iter().position(|held| held == name)
}
