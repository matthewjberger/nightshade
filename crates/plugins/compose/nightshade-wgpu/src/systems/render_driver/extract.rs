use crate::renderer::wgpu::WgpuRenderer;
use crate::renderer::wgpu::pass_sync;
use nightshade_text::contract::components::TextProperties;
use nightshade_text::contract::data::text_mesh::text_mesh_params;

pub(crate) fn prepare_text_meshes(
    renderer: &mut WgpuRenderer,
    storage: &mut nightshade::ecs::Storage,
    resources: &mut nightshade::ecs::ResourceMap,
) {
    let pre_revision = pass_sync::glyph_atlas_revision(renderer);

    let mut dirty_entries: Vec<(nightshade::ecs::Entity, usize, TextProperties)> = Vec::new();
    for (entity, text) in storage
        .query_ref::<&nightshade_text::contract::components::Text>()
        .iter()
    {
        if text.dirty {
            dirty_entries.push((entity, text.text_index, text.properties.clone()));
        }
    }
    for (entity, text_index, properties) in dirty_entries {
        let text_content = match resources
            .get::<nightshade_text::contract::resources::TextState>()
            .and_then(|state| state.cache.get_text(text_index))
        {
            Some(content) => content.to_string(),
            None => continue,
        };
        let params = text_mesh_params(&properties);
        let Some(text_state) =
            resources.get_mut::<nightshade_text::contract::resources::TextState>()
        else {
            continue;
        };
        let mesh = pass_sync::generate_world_text_mesh(
            renderer,
            &mut text_state.font_engine,
            &text_content,
            &params,
        );
        if let Some(text) = storage.get_mut::<nightshade_text::contract::components::Text>(entity) {
            text.cached_mesh = Some(mesh);
            text.dirty = false;
        }
    }

    pass_sync::publish_glyph_atlas_if_grown(renderer, pre_revision);
}
