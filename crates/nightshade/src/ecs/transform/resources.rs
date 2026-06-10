use freecs::Entity;
use std::collections::{HashMap, HashSet};

#[derive(Default, Clone)]
pub struct TransformState {
    pub dirty_entities: Vec<Entity>,
    pub children_cache: HashMap<Entity, Vec<Entity>>,
    pub children_cache_valid: bool,
    pub expansion_seen: HashSet<Entity>,
    pub ancestor_visited: HashSet<Entity>,
    pub render_dirty_scratch: Vec<Entity>,
}
