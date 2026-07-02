use freecs::Entity;
use std::collections::{HashMap, HashSet};

#[derive(Default, Clone)]
pub struct TransformState {
    pub children_cache: HashMap<Entity, Vec<Entity>>,
    pub children_cache_valid: bool,
    pub last_consumed_tick: u32,
    pub initial_pass_done: bool,
    pub expansion_seen: HashSet<Entity>,
    pub dirty_scratch: Vec<Entity>,
    pub traversal_stack: Vec<Entity>,
    pub render_dirty_scratch: Vec<Entity>,
}
