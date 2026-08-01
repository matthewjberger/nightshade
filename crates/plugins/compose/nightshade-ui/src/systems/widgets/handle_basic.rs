use crate::data::state::UiStateTrait as _;
use nalgebra_glm;
use nalgebra_glm::Vec2;
use nightshade::ecs::system_param::{Query, Res, ResMut};

pub type UiNodes<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item mut crate::components::UiLayoutNode,)>;

pub type UiColors<'world, 'item> =
    nightshade::ecs::system_param::Query<'world, (&'item mut crate::components::UiNodeColor,)>;

pub fn ui_toggle_system(
    timing: Res<crate::resources::RetainedUiTiming>,
    theme: Res<crate::resources::ThemeState>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    toggles: Query<(
        &mut crate::components::UiToggleData,
        &crate::components::UiNodeInteraction,
    )>,
    mut nodes: UiNodes<'_, '_>,
    mut colors: UiColors<'_, '_>,
) {
    let delta_time = timing.delta_time;
    let theme = theme.active_theme();
    let toggle_height = theme.toggle_height;
    let toggle_width = theme.toggle_width;
    let on_color = theme.accent_color;
    let off_color = theme.background_color;

    let mut pending = Vec::new();
    toggles.for_each(|entity, (data, interaction)| {
        let mut value = data.value;
        let mut changed = false;
        let mut animated_position = data.animated_position;

        if interaction.clicked {
            value = !value;
            changed = true;
        }

        let target = if value { 1.0 } else { 0.0 };
        let difference = target - animated_position;
        if difference.abs() > 0.001 {
            animated_position += difference.signum() * 8.0 * delta_time;
            animated_position = animated_position.clamp(0.0, 1.0);
        } else {
            animated_position = target;
        }

        data.value = value;
        data.changed = changed;
        data.animated_position = animated_position;
        pending.push((entity, data.knob_entity, value, changed, animated_position));
    });

    let knob_padding = 2.0;
    let knob_size = toggle_height - knob_padding * 2.0;
    let knob_travel = toggle_width - knob_size - knob_padding * 2.0;

    for (entity, knob_entity, value, changed, animated_position) in pending {
        let knob_x = knob_padding + animated_position * knob_travel;
        nodes.for_entity(knob_entity, |(node,)| {
            if let Some(crate::components::layout_types::UiLayoutType::Window(window)) =
                node.base_layout.as_mut()
            {
                window.position = crate::data::units::Ab(Vec2::new(knob_x, knob_padding)).into();
            }
        });
        let blended = off_color + (on_color - off_color) * animated_position;
        colors.for_entity(entity, |(color,)| {
            color.colors[crate::data::state::UiBase::INDEX] = Some(blended);
        });
        if changed {
            frame
                .events
                .push(crate::resources::UiEvent::ToggleChanged { entity, value });
        }
    }
}

pub fn ui_checkbox_system(
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    checkboxes: Query<(
        &mut crate::components::UiCheckboxData,
        &crate::components::UiNodeInteraction,
    )>,
    mut nodes: UiNodes<'_, '_>,
) {
    let mut pending = Vec::new();
    checkboxes.for_each(|entity, (data, interaction)| {
        if interaction.clicked {
            data.value = !data.value;
            data.changed = true;
            pending.push((entity, data.inner_entity, data.value));
        } else {
            data.changed = false;
        }
    });

    for (entity, inner_entity, value) in pending {
        nodes.for_entity(inner_entity, |(node,)| node.visible = value);
        frame
            .events
            .push(crate::resources::UiEvent::CheckboxChanged { entity, value });
    }
}

pub fn ui_radio_system(
    groups: Res<crate::resources::RetainedUiGroups>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut radios: Query<(
        &mut crate::components::UiRadioData,
        &crate::components::UiNodeInteraction,
    )>,
    mut nodes: UiNodes<'_, '_>,
) {
    let mut chosen = None;
    radios.for_each_mut(|entity, (data, interaction)| {
        data.changed = false;
        if interaction.clicked && !data.selected && chosen.is_none() {
            chosen = Some((entity, data.inner_entity, data.group_id, data.option_index));
        }
    });

    let Some((entity, inner_entity, group_id, option_index)) = chosen else {
        return;
    };

    nodes.for_entity(inner_entity, |(node,)| node.visible = true);

    let siblings = groups.radio.get(&group_id).cloned().unwrap_or_default();
    let mut deselected = Vec::new();
    for other_entity in siblings {
        if other_entity == entity {
            continue;
        }
        radios.for_entity(other_entity, |(other, _)| {
            if other.selected {
                other.selected = false;
                deselected.push(other.inner_entity);
            }
        });
    }
    for inner in deselected {
        nodes.for_entity(inner, |(node,)| node.visible = false);
    }

    radios.for_entity(entity, |(data, _)| {
        data.selected = true;
        data.changed = true;
    });
    frame.events.push(crate::resources::UiEvent::RadioChanged {
        entity,
        group_id,
        option_index,
    });
}

pub fn ui_collapsing_header_system(
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    headers: Query<(
        &mut crate::components::UiCollapsingHeaderData,
        &crate::components::UiNodeInteraction,
    )>,
    mut nodes: UiNodes<'_, '_>,
) {
    let mut pending = Vec::new();
    headers.for_each(|_entity, (data, interaction)| {
        if interaction.clicked {
            data.open = !data.open;
            data.changed = true;
            pending.push((data.content_entity, data.arrow_text_slot, data.open));
        } else {
            data.changed = false;
        }
    });

    for (content_entity, arrow_text_slot, open) in pending {
        nodes.for_entity(content_entity, |(node,)| node.visible = open);
        text_state
            .cache
            .set_text(arrow_text_slot, if open { "\u{25BC}" } else { "\u{25B6}" });
    }
}

pub fn ui_spinner_system(
    timing: Res<crate::resources::RetainedUiTiming>,
    theme: Res<crate::resources::ThemeState>,
    mut dirty: ResMut<crate::resources::RetainedUiDirty>,
    spinners: Query<(&mut crate::components::UiSpinnerData,)>,
    mut colors: UiColors<'_, '_>,
) {
    let delta_time = timing.delta_time;
    let accent = theme.active_theme().accent_color;

    let mut pending = Vec::new();
    spinners.for_each(|_entity, (data,)| {
        if data.dots.is_empty() {
            return;
        }
        let dot_count = data.dots.len() as f32;
        data.phase = (data.phase + delta_time * data.rotation_speed).rem_euclid(dot_count);
        pending.push((data.dots.clone(), data.phase));
    });

    if pending.is_empty() {
        return;
    }

    for (dots, head) in pending {
        let dot_count = dots.len() as f32;
        for (index, dot) in dots.iter().enumerate() {
            let offset = (head - index as f32).rem_euclid(dot_count);
            let alpha = 0.18 + (1.0 - offset / dot_count) * 0.82;
            colors.for_entity(*dot, |(color,)| {
                color.colors[crate::data::state::UiBase::INDEX] = Some(nalgebra_glm::Vec4::new(
                    accent.x,
                    accent.y,
                    accent.z,
                    alpha * accent.w,
                ));
            });
        }
    }
    dirty.render_dirty = true;
}
