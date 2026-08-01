use crate::theme;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct SavedFighter {
    pub slot: usize,
    pub experience: u32,
    pub rapport: u32,
    pub morale: i32,
    pub wounded: u32,
    pub kills: u32,
    pub battles: u32,
    pub arc_stage: u32,
    pub marks: u32,
    pub joined: bool,
    pub fallen: bool,
    pub deployed: bool,
    pub taken: Vec<usize>,
    pub bonds: Vec<(usize, u32)>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Save {
    pub chapter: usize,
    pub day: u32,
    pub actions: u32,
    pub supplies: i32,
    pub fighters: Vec<SavedFighter>,
}

impl nightshade_save::Save for Save {
    const FOLDER: &'static str = theme::SAVE_FOLDER;
    const NAME: &'static str = theme::SAVE_NAME;
    const VERSION: u32 = theme::SAVE_VERSION;
}
