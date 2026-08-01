use crate::theme;
use blocker_blocks::data::blocks::Palette;
use blocker_blocks::data::crops::CropKind;
use blocker_blocks::data::items::ItemStack;
use blocker_blocks::resources::Edits;
use nightshade_save::prelude::Autosave;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct SavedCrop {
    pub kind: CropKind,
    pub block: [i32; 3],
    pub progress: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SavedBoat {
    pub place: [f32; 3],
    pub yaw: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Saved {
    pub seed: u32,
    pub position: [f32; 3],
    pub home: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub health: f32,
    pub food: f32,
    pub slots: Vec<Option<ItemStack>>,
    pub selected: usize,
    pub chests: HashMap<[i32; 3], Vec<Option<ItemStack>>>,
    pub edited: Edits,
    pub crops: Vec<SavedCrop>,
    pub boats: Vec<SavedBoat>,
    pub palette: Palette,
}

impl nightshade_save::Save for Saved {
    const FOLDER: &'static str = theme::SAVE_FOLDER;
    const NAME: &'static str = theme::SAVE_NAME;
    const VERSION: u32 = theme::SAVE_VERSION;
    const CODEC: nightshade_save::SaveCodec = nightshade_save::SaveCodec::Bincode;
}

pub struct Save {
    pub found: Option<Saved>,
    pub autosave: Autosave,
}

impl Default for Save {
    fn default() -> Self {
        Self {
            found: None,
            autosave: Autosave::every(theme::SAVE_EVERY),
        }
    }
}
