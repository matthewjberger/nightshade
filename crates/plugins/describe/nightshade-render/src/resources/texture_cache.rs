use std::collections::HashMap;

use nightshade::ecs::Entity;

use crate::data::asset_id::TextureId;
use crate::data::generational_registry::*;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TextureOwner {
    EntityMaterial(Entity),
    EntityVariants(Entity),
    Named(String),
}

#[derive(Default)]
pub struct TextureCache {
    pub registry: GenerationalRegistry<()>,
    pub owners: HashMap<TextureOwner, Vec<TextureId>>,
}
