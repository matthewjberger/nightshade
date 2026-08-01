use nightshade::ecs::system_param::ResMut;

pub fn render_sync_transforms_system(
    mut transform_state: ResMut<nightshade_scene::resources::transform::TransformState>,
    mut mesh_state: ResMut<nightshade_render::resources::mesh_state::MeshRenderState>,
) {
    let mut moved = std::mem::take(&mut transform_state.moved_entities);
    for entity in moved.drain(..) {
        mesh_state.transform_dirty.push(entity);
    }
    transform_state.moved_entities = moved;
}
