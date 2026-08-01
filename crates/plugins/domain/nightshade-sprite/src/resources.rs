use crate::contract::render_data::{SpriteAtlasSpec, SpriteAtlasUpload};
use crate::contract::{SpriteFilter, SpriteRegion};
use crate::data::ShelfPacker;
use std::collections::HashMap;

pub struct SpriteAtlas {
    pub name: String,
    pub size: u32,
    pub filter: SpriteFilter,
    pub(crate) packer: ShelfPacker,
    pub(crate) regions: HashMap<String, SpriteRegion>,
    pub(crate) pending: Vec<SpriteAtlasUpload>,
}

#[derive(Default)]
pub struct SpriteAtlases {
    pub(crate) atlases: Vec<SpriteAtlas>,
    pub(crate) by_name: HashMap<String, u32>,
}

impl SpriteAtlases {
    pub fn index_of(&self, name: &str) -> Option<u32> {
        self.by_name.get(name).copied()
    }

    pub fn get(&self, name: &str) -> Option<&SpriteAtlas> {
        self.index_of(name)
            .and_then(|index| self.atlases.get(index as usize))
    }

    pub fn specs(&self) -> impl Iterator<Item = SpriteAtlasSpec> + '_ {
        self.atlases
            .iter()
            .enumerate()
            .map(|(index, atlas)| SpriteAtlasSpec {
                index: index as u32,
                size: atlas.size,
                filter: atlas.filter,
            })
    }
}
