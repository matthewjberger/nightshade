use nightshade_ecs::Entity;
use std::collections::HashSet;

#[derive(Default)]
pub struct Scratch {
    pub transform_dirty: Vec<Entity>,
    pub local_matrices_dirty: Vec<Entity>,
    pub instanced_seen: HashSet<Entity>,
}
