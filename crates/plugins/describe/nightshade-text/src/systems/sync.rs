use crate::contract::{Text, TextCharacterColors};
use crate::resources::TextSyncState;
use nightshade::ecs::Entity;
use nightshade::prelude::*;

pub fn mark_text_sync_dirty_system(
    mut state: ResMut<TextSyncState>,
    mesh_state: Res<nightshade_render::resources::mesh_state::MeshRenderState>,
    changed: Query<(&Text,), Changed<Text>>,
    recolored: Query<(&TextCharacterColors,), Changed<TextCharacterColors>>,
    revealed: Query<
        (&Text, &nightshade_scene::contract::primitives::Visibility),
        Changed<nightshade_scene::contract::primitives::Visibility>,
    >,
    mut text_lookup: Query<(&Text,)>,
) {
    let mut dirty = false;
    changed.for_each(|_entity, _| dirty = true);
    recolored.for_each(|_entity, _| dirty = true);
    revealed.for_each(|_entity, _| dirty = true);

    if !dirty {
        for entity in mesh_state.transform_dirty.iter().copied() {
            text_lookup.for_entity(entity, |_| dirty = true);
            if dirty {
                break;
            }
        }
    }

    state.dirty = dirty;
}

pub fn render_sync_text_system(
    mut state: ResMut<TextSyncState>,
    text_state: Res<crate::contract::TextState>,
    mut commands: Commands,
    texts: Query<(
        &Text,
        &nightshade_scene::contract::primitives::Visibility,
        &nightshade_scene::contract::GlobalTransform,
        Option<&TextCharacterColors>,
    )>,
    rendered: Query<(&crate::contract::RenderText,)>,
) {
    let mut count = 0;
    let mut collected: Vec<(Entity, crate::contract::RenderText)> = Vec::new();
    texts.for_each(|entity, (text, visibility, transform, character_colors)| {
        count += 1;
        if !visibility.visible {
            return;
        }
        let Some(mesh) = text.cached_mesh.clone() else {
            return;
        };
        let char_count = text_state
            .cache
            .get_text(text.text_index)
            .map(|content| {
                content
                    .chars()
                    .filter(|character| *character != '\n')
                    .count()
            })
            .unwrap_or(0);
        collected.push((
            entity,
            crate::contract::RenderText {
                mesh,
                color: text.properties.color,
                outline_color: text.properties.outline_color,
                outline_width: text.properties.outline_width,
                smoothing: text.properties.smoothing,
                billboard: text.billboard,
                transform: transform.0,
                char_count,
                character_colors: character_colors.map(|colors| colors.colors.clone()),
            },
        ));
    });

    let count_changed = count != state.entity_count;
    let dirty = state.dirty;
    state.entity_count = count;
    state.dirty = false;

    if !dirty && !count_changed {
        return;
    }

    rendered.for_each(|entity, _| {
        commands.remove::<(crate::contract::RenderText,)>(entity);
    });
    for (entity, render_text) in collected {
        commands.set(entity, render_text);
    }
}
