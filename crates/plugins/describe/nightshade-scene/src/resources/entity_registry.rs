use nightshade_ecs::Entity;

use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct EntityRegistry {
    pub tags: HashMap<Entity, Vec<String>>,
    pub next_guid: u64,
    pub guid_index: HashMap<u64, Entity>,
}
