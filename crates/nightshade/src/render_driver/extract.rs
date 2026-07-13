use crate::ecs::world::CORE;
use crate::render::wgpu::WgpuRenderer;
use crate::render::wgpu::pass_sync::rebind_glyph_atlas;

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

/// Regenerates dirty world-text meshes and the retained-UI text meshes for
/// this frame, rasterizing any new glyphs into the atlas. Runs before the
/// frame's inputs are composed, while the world still owns the retained
/// frame.
pub(crate) fn prepare_text_meshes(
    renderer: &mut WgpuRenderer,
    world: &mut crate::ecs::world::World,
) {
    let pre_revision = renderer.glyph_atlas.revision;

    let mut dirty_entries: Vec<(
        freecs::Entity,
        usize,
        crate::ecs::text::components::TextProperties,
    )> = Vec::new();
    for (entity, text) in world.ecs.worlds[CORE]
        .query_ref::<&crate::ecs::text::components::Text>()
        .iter()
    {
        if text.dirty {
            dirty_entries.push((entity, text.text_index, text.properties.clone()));
        }
    }
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
        if let Some(text) =
            world.ecs.worlds[CORE].get_mut::<crate::ecs::text::components::Text>(entity)
        {
            text.cached_mesh = Some(mesh);
            text.dirty = false;
        }
    }

    let mut frame_text = std::mem::take(&mut world.resources.retained_ui.frame.text_meshes);
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
    world.resources.retained_ui.frame.text_meshes = frame_text;

    if renderer.glyph_atlas.revision != pre_revision {
        rebind_glyph_atlas(renderer);
    }
}
