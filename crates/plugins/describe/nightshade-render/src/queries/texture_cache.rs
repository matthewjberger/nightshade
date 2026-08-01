use crate::data::asset_id::TextureId;
use crate::data::generational_registry::*;
use crate::resources::texture_cache::TextureCache;

pub fn texture_cache_texture_id(cache: &TextureCache, name: &str) -> Option<TextureId> {
    registry_lookup_index(&cache.registry, name)
        .map(|(index, generation)| TextureId::new(index, generation))
}

pub fn texture_cache_texture_name(cache: &TextureCache, index: u32) -> Option<&str> {
    registry_name_for(&cache.registry, index)
}
