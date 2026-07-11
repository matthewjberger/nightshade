use crate::render::wgpu::WgpuRenderer;
use crate::render::wgpu::passes;
use crate::render::wgpu::rendergraph::render_graph_get_pass_mut;

pub(crate) fn extract_frame_dirty_state(
    renderer: &mut WgpuRenderer,
    world: &mut crate::ecs::world::World,
    inputs: &mut crate::render::wgpu::render_configs::RenderInputs,
) {
    use crate::render::mesh_cache::{mesh_cache_take_dirty, mesh_cache_take_dirty_skinned};

    let frame_state = world.resources.mesh_render_state.take_frame_state();
    let dirty_meshes = mesh_cache_take_dirty(&mut inputs.mesh_cache);
    let dirty_skinned = mesh_cache_take_dirty_skinned(&mut inputs.mesh_cache);

    let shadow_scene_dirty = !frame_state.transform_dirty.is_empty()
        || !frame_state.entities_added.is_empty()
        || !frame_state.entities_removed.is_empty()
        || frame_state.instanced_meshes_changed
        || !dirty_meshes.is_empty()
        || !dirty_skinned.is_empty();

    if let Some(shadow_pass) = render_graph_get_pass_mut(&mut renderer.graph, "shadow_depth_pass")
        && let Some(pass) =
            (shadow_pass as &mut dyn std::any::Any).downcast_mut::<passes::ShadowDepthPass>()
    {
        pass.shadow_scene_dirty = shadow_scene_dirty;
    }

    if let Some(mesh_pass) = render_graph_get_pass_mut(&mut renderer.graph, "mesh_pass")
        && let Some(pass) = (mesh_pass as &mut dyn std::any::Any).downcast_mut::<passes::MeshPass>()
    {
        pass.frame_dirty = Some(frame_state);
        let dirty_ids: std::collections::HashSet<u32> = dirty_meshes
            .iter()
            .filter_map(|name| pass.mesh_id_for_name(name))
            .collect();
        pass.dirty_mesh_ids = dirty_ids;
    }

    if let Some(skinned_pass) = render_graph_get_pass_mut(&mut renderer.graph, "skinned_mesh_pass")
        && let Some(pass) = (skinned_pass as &mut dyn std::any::Any)
            .downcast_mut::<passes::geometry::SkinnedMeshPass>()
    {
        pass.dirty_skinned_mesh_names.extend(dirty_skinned);
    }
}

fn compute_ui_text_signature(text_instance: &crate::render::ui_data::UiTextInstance) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text_instance.text.hash(&mut hasher);
    text_instance.font_size.to_bits().hash(&mut hasher);
    (text_instance.alignment as u8).hash(&mut hasher);
    (text_instance.vertical_alignment as u8).hash(&mut hasher);
    text_instance.monospace.hash(&mut hasher);
    text_instance
        .wrap_width
        .map(|w| w.to_bits())
        .unwrap_or(u32::MAX)
        .hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn prepare_text_meshes(
    renderer: &mut WgpuRenderer,
    world: &mut crate::ecs::world::World,
    inputs: &mut crate::render::wgpu::render_configs::RenderInputs,
) {
    let pre_revision = renderer.glyph_atlas.revision;

    let mut dirty_entries: Vec<(
        freecs::Entity,
        usize,
        crate::ecs::text::components::TextProperties,
    )> = Vec::new();
    world
        .core
        .query()
        .with(crate::ecs::world::TEXT)
        .iter(|entity, table, idx| {
            let text = &table.text[idx];
            if text.dirty {
                dirty_entries.push((entity, text.text_index, text.properties.clone()));
            }
        });
    for (entity, text_index, properties) in dirty_entries {
        let text_content = match world.resources.text.cache.get_text(text_index) {
            Some(t) => t.to_string(),
            None => continue,
        };
        let mesh = super::text_mesh::generate_text_mesh_world(
            &text_content,
            &mut super::text_mesh::TextMeshContext {
                font_engine: &mut world.resources.text.font_engine,
                glyph_atlas: &mut renderer.glyph_atlas,
                device: &renderer.device,
                queue: &renderer.queue,
            },
            &properties,
        );
        if let Some(text) = world.core.get_text_mut(entity) {
            text.cached_mesh = Some(mesh);
            text.dirty = false;
        }
    }

    let mut frame_text = std::mem::take(&mut inputs.ui.text_meshes);
    let mut active_entities: std::collections::HashSet<freecs::Entity> =
        std::collections::HashSet::new();
    for instance in &mut frame_text {
        let signature = compute_ui_text_signature(instance);
        let cached = instance
            .entity
            .and_then(|e| renderer.frame_state.text_mesh_signatures.get(&e).copied());
        if cached == Some(signature) {
            if let Some(entity) = instance.entity {
                active_entities.insert(entity);
            }
            continue;
        }
        let properties = crate::ecs::text::components::TextProperties {
            font_size: instance.font_size,
            color: instance.color,
            alignment: instance.alignment,
            vertical_alignment: instance.vertical_alignment,
            line_height: 1.2,
            outline_color: instance.outline_color,
            outline_width: instance.outline_width,
            monospace_width: if instance.monospace { Some(0.0) } else { None },
            font_kind: instance.font_kind,
            ..crate::ecs::text::components::TextProperties::default()
        };
        instance.mesh = super::text_mesh::generate_text_mesh_ui(
            &instance.text,
            &mut super::text_mesh::TextMeshContext {
                font_engine: &mut world.resources.text.font_engine,
                glyph_atlas: &mut renderer.glyph_atlas,
                device: &renderer.device,
                queue: &renderer.queue,
            },
            &properties,
            instance.wrap_width,
        );
        if let Some(entity) = instance.entity {
            renderer
                .frame_state
                .text_mesh_signatures
                .insert(entity, signature);
            active_entities.insert(entity);
        }
    }
    let stale: Vec<freecs::Entity> = renderer
        .frame_state
        .text_mesh_signatures
        .keys()
        .copied()
        .filter(|e| !active_entities.contains(e))
        .collect();
    for entity in stale {
        renderer.frame_state.text_mesh_signatures.remove(&entity);
    }
    inputs.ui.text_meshes = frame_text;

    if renderer.glyph_atlas.revision != pre_revision {
        if let Some(overlay_pass) =
            render_graph_get_pass_mut(&mut renderer.graph, "scene_overlay_pass")
            && let Some(overlay_pass) =
                (overlay_pass as &mut dyn std::any::Any).downcast_mut::<passes::SceneOverlayPass>()
        {
            overlay_pass
                .text
                .update_glyph_atlas(&renderer.device, &renderer.glyph_atlas.texture_view);
        }
        if let Some(ui_pass) = render_graph_get_pass_mut(&mut renderer.graph, "ui_pass")
            && let Some(ui_pass) =
                (ui_pass as &mut dyn std::any::Any).downcast_mut::<passes::UiPass>()
        {
            ui_pass.update_glyph_atlas(&renderer.device, &renderer.glyph_atlas.texture_view);
        }
    }
}
