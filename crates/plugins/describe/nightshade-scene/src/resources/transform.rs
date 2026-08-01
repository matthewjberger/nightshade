use nightshade_ecs::Entity;
use std::collections::HashSet;

#[derive(Default, Clone)]
pub struct TransformState {
    pub last_consumed_tick: u32,
    pub initial_pass_done: bool,
    pub expansion_seen: HashSet<Entity>,
    pub dirty_scratch: Vec<Entity>,
    pub traversal_stack: Vec<Entity>,
    pub moved_entities: Vec<Entity>,
}
