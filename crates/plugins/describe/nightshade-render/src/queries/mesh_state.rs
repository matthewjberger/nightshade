use crate::resources::mesh_state::MeshRenderState;
use nightshade::ecs::Entity;

pub fn mesh_state_has_changes(state: &MeshRenderState) -> bool {
    !state.transform_dirty.is_empty()
        || !state.material_dirty.is_empty()
        || !state.entities_added.is_empty()
        || !state.entities_removed.is_empty()
        || state.batches_invalidated
        || state.full_rebuild_needed
}

pub fn mesh_state_dirty_for_culling(state: &MeshRenderState) -> impl Iterator<Item = Entity> + '_ {
    state
        .transform_dirty
        .iter()
        .chain(state.material_dirty.iter())
        .chain(state.entities_added.iter())
        .chain(state.object_state_dirty.iter())
        .copied()
}

pub fn mesh_state_needs_full_invalidation(state: &MeshRenderState) -> bool {
    state.full_rebuild_needed || state.batches_invalidated || !state.entities_removed.is_empty()
}
