use crate::ecs::text::components::TextProperties;
use crate::render::text_mesh::TextMeshParams;
use crate::render::wgpu::WgpuRenderer;
use crate::render::wgpu::pass_sync;
use nightshade_text::text_data::FontKind;

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
    let pre_revision = pass_sync::glyph_atlas_revision(renderer);

    let mut dirty_entries: Vec<(
        nightshade_ecs::Entity,
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
        let mesh = pass_sync::generate_world_text_mesh(
            renderer,
            &mut world
                .res_mut::<crate::ecs::text::resources::TextState>()
                .font_engine,
            &text_content,
            &params,
        );
        if let Some(text) = world.get_mut::<crate::ecs::text::components::Text>(entity) {
            text.cached_mesh = Some(mesh);
            text.dirty = false;
        }
    }

    let mut frame_text = std::mem::take(
        &mut world
            .res_mut::<crate::ui::resources::RetainedUiFrame>()
            .text_meshes,
    );
    let mut active_entities: std::collections::HashSet<nightshade_ecs::Entity> =
        std::collections::HashSet::new();
    for instance in &mut frame_text {
        let signature = compute_ui_text_signature(instance);
        let cached = instance
            .entity
            .and_then(|e| pass_sync::cached_ui_text_signature(renderer, e));
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
        instance.mesh = pass_sync::generate_ui_text_mesh(
            renderer,
            &mut world
                .res_mut::<crate::ecs::text::resources::TextState>()
                .font_engine,
            &instance.text,
            &params,
            instance.wrap_width,
        );
        if let Some(entity) = instance.entity {
            pass_sync::record_ui_text_signature(renderer, entity, signature);
            active_entities.insert(entity);
        }
    }
    pass_sync::retain_ui_text_signatures(renderer, &active_entities);
    world
        .res_mut::<crate::ui::resources::RetainedUiFrame>()
        .text_meshes = frame_text;

    pass_sync::rebind_glyph_atlas_if_grown(renderer, pre_revision);
}
