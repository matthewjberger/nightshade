use crate::ecs::scene::MetadataValue;
use freecs::Entity;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct EntityRegistry {
    pub names: HashMap<String, Entity>,
    pub tags: HashMap<Entity, Vec<String>>,
    pub metadata: HashMap<Entity, HashMap<String, MetadataValue>>,
}
