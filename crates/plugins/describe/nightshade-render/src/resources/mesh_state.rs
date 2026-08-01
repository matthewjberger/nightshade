use nightshade::ecs::Entity;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct MeshRenderState {
    pub transform_dirty: Vec<Entity>,
    pub material_dirty: Vec<Entity>,
    pub instanced_local_matrices_dirty: Vec<Entity>,
    pub custom_data_overrides: HashMap<Entity, [f32; 4]>,
    pub custom_data_dirty: Vec<Entity>,
    pub object_state_dirty: Vec<Entity>,
    pub instanced_compute_dispatch_needed: bool,
    pub instanced_meshes_changed: bool,
    pub entities_added: Vec<Entity>,
    pub entities_removed: Vec<Entity>,
    pub batches_invalidated: bool,
    pub full_rebuild_needed: bool,
    pub frame_initialized: bool,
}
