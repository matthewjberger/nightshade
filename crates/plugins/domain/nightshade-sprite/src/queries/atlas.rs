use crate::contract::SpriteRegion;
use crate::resources::SpriteAtlases;

pub fn sprite_region(atlases: &SpriteAtlases, atlas: &str, name: &str) -> Option<SpriteRegion> {
    atlases.get(atlas)?.regions.get(name).copied()
}
