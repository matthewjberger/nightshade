use crate::ecs::text::components::TextProperties;
use crate::render::text_data::FontKind;
use crate::render::text_mesh::TextMeshParams;
use crate::render::wgpu::WgpuRenderer;
use crate::render::wgpu::pass_sync::rebind_glyph_atlas;

/// Resolves a text component into the renderer's shaping parameters, collapsing
/// the monospace request onto the mono font and leaving the icon fonts as chosen.
/// The renderer's mesher never names the engine's text component, so this is the
/// one place the two vocabularies meet.
fn text_mesh_params(properties: &TextProperties) -> TextMeshParams {
    let font_kind = match properties.font_kind {
        FontKind::IconsMaterial | FontKind::IconsLucide => properties.font_kind,
        _ if properties.monospace_width.is_some() => FontKind::Mono,
        kind => kind,
    };
    TextMeshParams {
        font_size: properties.font_size,
        line_height: properties.line_height,
        font_kind,
        alignment: properties.alignment,
        vertical_alignment: properties.vertical_alignment,
        anchor_character: properties.anchor_character,
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
    for (entity, text) in world
        .query_ref::<&crate::ecs::text::components::Text>()
        .iter()
    {
        if text.dirty {
            dirty_entries.push((entity, text.text_index, text.properties.clone()));
        }
    }
    for (entity, text_index, properties) in dirty_entries {
        let text_content = match world
            .res::<crate::ecs::text::resources::TextState>()
            .cache
            .get_text(text_index)
        {
            Some(t) => t.to_string(),
            None => continue,
        };
        let params = text_mesh_params(&properties);
        let mesh = crate::render::text_mesh::generate_text_mesh_world(
            &text_content,
            &mut crate::render::text_mesh::TextMeshContext {
                font_engine: &mut world
                    .res_mut::<crate::ecs::text::resources::TextState>()
                    .font_engine,
                glyph_atlas: &mut renderer.glyph_atlas,
                device: &renderer.device,
                queue: &renderer.queue,
            },
            &params,
        );
        if let Some(text) = world.get_mut::<crate::ecs::text::components::Text>(entity) {
            text.cached_mesh = Some(mesh);
            text.dirty = false;
        }
    }

    let mut frame_text = std::mem::take(
        &mut world
            .res_mut::<crate::ecs::ui::resources::RetainedUiState>()
            .frame
            .text_meshes,
    );
    let mut active_entities: std::collections::HashSet<crate::render::entity::RenderEntity> =
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
        let properties = TextProperties {
            font_size: instance.font_size,
            color: instance.color,
            alignment: instance.alignment,
            vertical_alignment: instance.vertical_alignment,
            line_height: 1.2,
            outline_color: instance.outline_color,
            outline_width: instance.outline_width,
            monospace_width: if instance.monospace { Some(0.0) } else { None },
            font_kind: instance.font_kind,
            ..TextProperties::default()
        };
        let params = text_mesh_params(&properties);
        instance.mesh = crate::render::text_mesh::generate_text_mesh_ui(
            &instance.text,
            &mut crate::render::text_mesh::TextMeshContext {
                font_engine: &mut world
                    .res_mut::<crate::ecs::text::resources::TextState>()
                    .font_engine,
                glyph_atlas: &mut renderer.glyph_atlas,
                device: &renderer.device,
                queue: &renderer.queue,
            },
            &params,
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
    let stale: Vec<crate::render::entity::RenderEntity> = renderer
        .frame_state
        .text_mesh_signatures
        .keys()
        .copied()
        .filter(|e| !active_entities.contains(e))
        .collect();
    for entity in stale {
        renderer.frame_state.text_mesh_signatures.remove(&entity);
    }
    world
        .res_mut::<crate::ecs::ui::resources::RetainedUiState>()
        .frame
        .text_meshes = frame_text;

    if renderer.glyph_atlas.revision != pre_revision {
        rebind_glyph_atlas(renderer);
    }
}
