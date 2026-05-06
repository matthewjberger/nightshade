use freecs::Entity;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct TransformState {
    pub dirty_entities: Vec<Entity>,
    pub children_cache: HashMap<Entity, Vec<Entity>>,
    pub children_cache_valid: bool,
}
