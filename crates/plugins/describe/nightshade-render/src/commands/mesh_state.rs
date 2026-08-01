use crate::resources::mesh_state::MeshRenderState;
use nightshade::ecs::Entity;

pub fn mesh_state_mark_material_dirty(state: &mut MeshRenderState, entity: Entity) {
    state.material_dirty.push(entity);
    state.batches_invalidated = true;
}

pub fn mesh_state_mark_entity_added(state: &mut MeshRenderState, entity: Entity) {
    state.entities_added.push(entity);
    state.batches_invalidated = true;
}

pub fn mesh_state_mark_entity_removed(state: &mut MeshRenderState, entity: Entity) {
    state.entities_removed.push(entity);
    state.batches_invalidated = true;
    state.transform_dirty.retain(|held| *held != entity);
    state.material_dirty.retain(|held| *held != entity);
}

pub fn mesh_state_set_custom_data(state: &mut MeshRenderState, entity: Entity, data: [f32; 4]) {
    let previous_alpha = state
        .custom_data_overrides
        .get(&entity)
        .map_or(1.0, |held| held[3]);
    if (previous_alpha < 1.0) != (data[3] < 1.0) {
        state.batches_invalidated = true;
    }
    state.custom_data_overrides.insert(entity, data);
    state.custom_data_dirty.push(entity);
}

pub fn mesh_state_fade_complete(state: &mut MeshRenderState, entity: Entity) {
    state
        .custom_data_overrides
        .insert(entity, [1.0, 1.0, 1.0, 1.0]);
    state.custom_data_dirty.push(entity);
    state.batches_invalidated = true;
}

pub fn mesh_state_take_frame(state: &mut MeshRenderState) -> MeshRenderState {
    std::mem::replace(
        state,
        MeshRenderState {
            frame_initialized: true,
            ..Default::default()
        },
    )
}

pub fn mesh_state_restore_unconsumed(state: &mut MeshRenderState, unconsumed: MeshRenderState) {
    let newer = std::mem::replace(state, unconsumed);
    mesh_state_merge_newer(state, newer);
}

pub fn mesh_state_merge_newer(state: &mut MeshRenderState, newer: MeshRenderState) {
    extend_once(&mut state.transform_dirty, newer.transform_dirty);
    extend_once(&mut state.material_dirty, newer.material_dirty);
    extend_once(
        &mut state.instanced_local_matrices_dirty,
        newer.instanced_local_matrices_dirty,
    );
    extend_once(&mut state.custom_data_dirty, newer.custom_data_dirty);
    extend_once(&mut state.object_state_dirty, newer.object_state_dirty);
    state
        .custom_data_overrides
        .extend(newer.custom_data_overrides);
    state.instanced_compute_dispatch_needed |= newer.instanced_compute_dispatch_needed;
    state.instanced_meshes_changed |= newer.instanced_meshes_changed;
    state.entities_added.extend(newer.entities_added);
    state.entities_removed.extend(newer.entities_removed);
    state.batches_invalidated |= newer.batches_invalidated;
    state.full_rebuild_needed |= newer.full_rebuild_needed;
    state.frame_initialized |= newer.frame_initialized;
}

fn extend_once(held: &mut Vec<Entity>, newer: Vec<Entity>) {
    held.extend(newer);
    held.sort_unstable();
    held.dedup();
}
