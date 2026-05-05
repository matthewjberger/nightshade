use freecs::Entity;

#[derive(Default)]
pub struct Scratch {
    pub transform_dirty: Vec<Entity>,
    pub local_matrices_dirty: Vec<Entity>,
}
