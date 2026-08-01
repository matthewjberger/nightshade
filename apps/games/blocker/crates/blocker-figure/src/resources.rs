use crate::data::characters::CHARACTERS;
use nightshade::ecs::Entity;
use nightshade_load::LoadHandle;
use nightshade_prefab::Prefab;

pub struct Wardrobe {
    pub handles: Vec<Option<LoadHandle>>,
    pub kits: Vec<Option<Prefab>>,
    pub requested: Vec<bool>,
    pub failed: Vec<bool>,
}

impl Default for Wardrobe {
    fn default() -> Self {
        let count = CHARACTERS.len();
        Self {
            handles: vec![None; count],
            kits: vec![None; count],
            requested: vec![false; count],
            failed: vec![false; count],
        }
    }
}

pub struct Nameplates {
    pub shown: bool,
}

impl Default for Nameplates {
    fn default() -> Self {
        Self { shown: true }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Hero(pub Option<Entity>);
