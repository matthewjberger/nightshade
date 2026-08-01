use nalgebra_glm;
use nalgebra_glm::Vec2;
use nightshade::ecs::system_param::{Query, Res, ResMut};
use nightshade_platform::KeyCode;

use super::handle_scroll::{UiInteractions, UiNodes};

pub type UiSliders<'world, 'item> = Query<'world, (&'item mut crate::components::UiSliderData,)>;

pub type UiSplitters<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiSplitterData,)>;

pub type UiBreadcrumbs<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiBreadcrumbData,)>;

pub type UiRangeSliders<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiRangeSliderData,)>;

#[derive(Default, Clone, Copy)]
pub(super) struct RowInteraction {
    pub pressed: bool,
    pub dragging: bool,
    pub clicked: bool,
    pub double_clicked: bool,
    pub hovered: bool,
    pub drag_start: Option<nalgebra_glm::Vec2>,
}

pub(super) fn row_interaction(
    interactions: &mut UiInteractions<'_, '_>,
    entity: nightshade::ecs::Entity,
) -> RowInteraction {
    let mut row = RowInteraction::default();
    interactions.for_entity(entity, |(interaction,)| {
        row.pressed = interaction.pressed;
        row.dragging = interaction.dragging;
        row.clicked = interaction.clicked;
        row.double_clicked = interaction.double_clicked;
        row.hovered = interaction.hovered;
        row.drag_start = interaction.drag_start;
    });
    row
}

pub(super) fn node_rect(
    nodes: &mut UiNodes<'_, '_>,
    entity: nightshade::ecs::Entity,
) -> Option<nightshade_paint::data::Rect> {
    let mut rect = None;
    nodes.for_entity(entity, |(node,)| rect = Some(node.computed_rect));
    rect
}

fn set_boundary_fill(nodes: &mut UiNodes<'_, '_>, entity: nightshade::ecs::Entity, percent: f32) {
    nodes.for_entity(entity, |(node,)| {
        if let Some(crate::components::layout_types::UiLayoutType::Boundary(boundary)) =
            node.base_layout.as_mut()
        {
            boundary.position_2 = crate::data::units::Rl(Vec2::new(percent, 100.0)).into();
        }
    });
}

pub fn ui_slider_system(
    interaction_state: Res<crate::resources::RetainedUiInteraction>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    input: Res<nightshade_platform::contract::input::Input>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    mut sliders: UiSliders<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
) {
    let mouse_position = input.mouse.position;
    let focused_entity = interaction_state.focused_entity;

    let mut pending = Vec::new();
    sliders.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        let interaction = row_interaction(&mut interactions, entity);
        let mut new_value = data.value;
        let mut changed = false;

        if focused_entity == Some(entity) {
            let range = data.max - data.min;
            let step = if data.logarithmic {
                range * 0.01
            } else {
                10.0_f32.powf((range.log10() - 1.0).floor()).max(0.01)
            };
            for (key, is_pressed) in &mirror.frame_keys {
                if !is_pressed {
                    continue;
                }
                match key {
                    KeyCode::ArrowRight | KeyCode::ArrowUp => {
                        new_value = (new_value + step).min(data.max);
                        changed = true;
                    }
                    KeyCode::ArrowLeft | KeyCode::ArrowDown => {
                        new_value = (new_value - step).max(data.min);
                        changed = true;
                    }
                    KeyCode::Home => {
                        new_value = data.min;
                        changed = true;
                    }
                    KeyCode::End => {
                        new_value = data.max;
                        changed = true;
                    }
                    _ => {}
                }
            }
        }

        let dragging = interaction.pressed || interaction.dragging;
        if dragging && let Some(rect) = node_rect(&mut nodes, entity) {
            let normalized = ((mouse_position.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
            new_value = if data.logarithmic && data.min > 0.0 && data.max > data.min {
                data.min * (data.max / data.min).powf(normalized)
            } else {
                data.min + normalized * (data.max - data.min)
            };
            if (new_value - data.value).abs() > f32::EPSILON {
                changed = true;
            }
            set_boundary_fill(&mut nodes, data.fill_entity, normalized * 100.0);
            text_state.cache.set_text(
                data.text_slot,
                format!(
                    "{}{:.prec$}{}",
                    data.prefix,
                    new_value,
                    data.suffix,
                    prec = data.precision
                ),
            );
        }

        if changed && !dragging {
            let range = data.max - data.min;
            let normalized = if data.logarithmic && data.min > 0.0 && range > 0.0 {
                ((new_value / data.min).ln() / (data.max / data.min).ln()).clamp(0.0, 1.0)
            } else if range.abs() > f32::EPSILON {
                ((new_value - data.min) / range).clamp(0.0, 1.0)
            } else {
                0.0
            };
            set_boundary_fill(&mut nodes, data.fill_entity, normalized * 100.0);
            text_state.cache.set_text(
                data.text_slot,
                format!(
                    "{}{:.prec$}{}",
                    data.prefix,
                    new_value,
                    data.suffix,
                    prec = data.precision
                ),
            );
        }

        sliders.for_entity(entity, |(widget_data,)| {
            widget_data.value = new_value;
            widget_data.changed = changed;
        });
        if changed {
            frame.events.push(crate::resources::UiEvent::SliderChanged {
                entity,
                value: new_value,
            });
        }
    }
}

pub fn ui_splitter_system(
    input: Res<nightshade_platform::contract::input::Input>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut splitters: UiSplitters<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
) {
    use crate::components::SplitDirection;

    let mouse_position = input.mouse.position;

    let mut pending = Vec::new();
    splitters.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        let divider_dragging = row_interaction(&mut interactions, data.divider_entity).dragging;
        if !divider_dragging {
            splitters.for_entity(entity, |(splitter,)| splitter.changed = false);
            continue;
        }

        let Some(container_rect) = node_rect(&mut nodes, entity) else {
            continue;
        };

        let new_ratio = match data.direction {
            SplitDirection::Horizontal => {
                let width = container_rect.width();
                if width > 0.0 {
                    ((mouse_position.x - container_rect.min.x) / width)
                        .clamp(data.min_ratio, data.max_ratio)
                } else {
                    data.ratio
                }
            }
            SplitDirection::Vertical => {
                let height = container_rect.height();
                if height > 0.0 {
                    ((mouse_position.y - container_rect.min.y) / height)
                        .clamp(data.min_ratio, data.max_ratio)
                } else {
                    data.ratio
                }
            }
        };

        if (new_ratio - data.ratio).abs() < 0.001 {
            continue;
        }

        let half = 4.0 * 0.5;
        let (
            first_pane_min,
            first_pane_max,
            divider_min,
            divider_max,
            second_pane_min,
            second_pane_max,
        ) = match data.direction {
            SplitDirection::Horizontal => (
                crate::data::units::Rl(Vec2::new(0.0, 0.0)),
                crate::data::units::Rl(Vec2::new(new_ratio * 100.0, 100.0))
                    + crate::data::units::Ab(Vec2::new(-half, 0.0)),
                crate::data::units::Rl(Vec2::new(new_ratio * 100.0, 0.0))
                    + crate::data::units::Ab(Vec2::new(-half, 0.0)),
                crate::data::units::Rl(Vec2::new(new_ratio * 100.0, 100.0))
                    + crate::data::units::Ab(Vec2::new(half, 0.0)),
                crate::data::units::Rl(Vec2::new(new_ratio * 100.0, 0.0))
                    + crate::data::units::Ab(Vec2::new(half, 0.0)),
                crate::data::units::Rl(Vec2::new(100.0, 100.0)),
            ),
            SplitDirection::Vertical => (
                crate::data::units::Rl(Vec2::new(0.0, 0.0)),
                crate::data::units::Rl(Vec2::new(100.0, new_ratio * 100.0))
                    + crate::data::units::Ab(Vec2::new(0.0, -half)),
                crate::data::units::Rl(Vec2::new(0.0, new_ratio * 100.0))
                    + crate::data::units::Ab(Vec2::new(0.0, -half)),
                crate::data::units::Rl(Vec2::new(100.0, new_ratio * 100.0))
                    + crate::data::units::Ab(Vec2::new(0.0, half)),
                crate::data::units::Rl(Vec2::new(0.0, new_ratio * 100.0))
                    + crate::data::units::Ab(Vec2::new(0.0, half)),
                crate::data::units::Rl(Vec2::new(100.0, 100.0)),
            ),
        };

        let mut set_boundary = |entity, position_1, position_2| {
            nodes.for_entity(entity, |(node,)| {
                if let Some(crate::components::layout_types::UiLayoutType::Boundary(boundary)) =
                    node.base_layout.as_mut()
                {
                    boundary.position_1 = position_1;
                    boundary.position_2 = position_2;
                }
            });
        };
        set_boundary(data.first_pane, first_pane_min.into(), first_pane_max);
        set_boundary(data.divider_entity, divider_min, divider_max);
        set_boundary(data.second_pane, second_pane_min, second_pane_max.into());

        splitters.for_entity(entity, |(splitter,)| {
            splitter.ratio = new_ratio;
            splitter.changed = true;
        });
        frame
            .events
            .push(crate::resources::UiEvent::SplitterChanged {
                entity,
                ratio: new_ratio,
            });
    }
}

pub fn ui_breadcrumb_system(
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut breadcrumbs: UiBreadcrumbs<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
) {
    let mut pending = Vec::new();
    breadcrumbs
        .for_each_mut(|entity, (data,)| pending.push((entity, data.segment_entities.clone())));

    for (entity, segments) in pending {
        let clicked_segment = segments
            .iter()
            .position(|segment_entity| row_interaction(&mut interactions, *segment_entity).clicked);

        match clicked_segment {
            Some(index) => {
                breadcrumbs.for_entity(entity, |(data,)| {
                    data.clicked_segment = Some(index);
                    data.changed = true;
                });
                frame
                    .events
                    .push(crate::resources::UiEvent::BreadcrumbClicked {
                        entity,
                        segment_index: index,
                    });
            }
            None => breadcrumbs.for_entity(entity, |(data,)| {
                data.clicked_segment = None;
                data.changed = false;
            }),
        }
    }
}

pub fn ui_range_slider_system(
    interaction_state: Res<crate::resources::RetainedUiInteraction>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    input: Res<nightshade_platform::contract::input::Input>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    mut range_sliders: UiRangeSliders<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
) {
    use crate::components::RangeSliderThumb;

    let mouse_position = input.mouse.position;
    let focused_entity = interaction_state.focused_entity;

    let mut pending = Vec::new();
    range_sliders.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        let interaction = row_interaction(&mut interactions, entity);
        let mut low_value = data.low_value;
        let mut high_value = data.high_value;
        let mut changed = false;
        let mut active_thumb = data.active_thumb;

        let range = data.max - data.min;
        if range.abs() < f32::EPSILON {
            range_sliders.for_entity(entity, |(widget_data,)| widget_data.changed = false);
            continue;
        }

        if interaction.pressed
            && let Some(rect) = node_rect(&mut nodes, entity)
        {
            let normalized = ((mouse_position.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
            let mouse_value = data.min + normalized * range;
            active_thumb = Some(
                if (mouse_value - low_value).abs() <= (mouse_value - high_value).abs() {
                    RangeSliderThumb::Low
                } else {
                    RangeSliderThumb::High
                },
            );
        }

        let dragging = interaction.pressed || interaction.dragging;
        if dragging
            && let Some(thumb) = active_thumb
            && let Some(rect) = node_rect(&mut nodes, entity)
        {
            let normalized = ((mouse_position.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
            let new_value = data.min + normalized * range;
            match thumb {
                RangeSliderThumb::Low => low_value = new_value.min(high_value),
                RangeSliderThumb::High => high_value = new_value.max(low_value),
            }
            if (low_value - data.low_value).abs() > f32::EPSILON
                || (high_value - data.high_value).abs() > f32::EPSILON
            {
                changed = true;
            }
            super::range_slider_visuals::update_range_slider_visuals(
                &mut nodes,
                &mut text_state,
                &super::range_slider_visuals::RangeSliderVisualUpdate {
                    fill_entity: data.fill_entity,
                    low_thumb: data.low_thumb_entity,
                    high_thumb: data.high_thumb_entity,
                    text_slot: data.text_slot,
                    precision: data.precision,
                    low_normalized: ((low_value - data.min) / range).clamp(0.0, 1.0),
                    high_normalized: ((high_value - data.min) / range).clamp(0.0, 1.0),
                    low_value,
                    high_value,
                    thumb_half: data.thumb_half_size,
                },
            );
        }

        if !dragging {
            active_thumb = None;
        }

        if focused_entity == Some(entity) {
            let step = range * 0.01;
            for &(key, pressed) in &mirror.frame_keys {
                if !pressed {
                    continue;
                }
                match key {
                    KeyCode::Tab => {
                        active_thumb = match active_thumb {
                            Some(RangeSliderThumb::Low) => Some(RangeSliderThumb::High),
                            _ => Some(RangeSliderThumb::Low),
                        };
                    }
                    KeyCode::ArrowLeft | KeyCode::ArrowDown => {
                        let thumb = active_thumb.unwrap_or(RangeSliderThumb::Low);
                        match thumb {
                            RangeSliderThumb::Low => {
                                low_value = (low_value - step).max(data.min).min(high_value)
                            }
                            RangeSliderThumb::High => {
                                high_value = (high_value - step).max(low_value).max(data.min)
                            }
                        }
                        if active_thumb.is_none() {
                            active_thumb = Some(thumb);
                        }
                        changed = true;
                    }
                    KeyCode::ArrowRight | KeyCode::ArrowUp => {
                        let thumb = active_thumb.unwrap_or(RangeSliderThumb::High);
                        match thumb {
                            RangeSliderThumb::Low => {
                                low_value = (low_value + step).min(high_value).min(data.max)
                            }
                            RangeSliderThumb::High => {
                                high_value = (high_value + step).min(data.max)
                            }
                        }
                        if active_thumb.is_none() {
                            active_thumb = Some(thumb);
                        }
                        changed = true;
                    }
                    _ => {}
                }
            }

            if changed && !dragging {
                super::range_slider_visuals::update_range_slider_visuals(
                    &mut nodes,
                    &mut text_state,
                    &super::range_slider_visuals::RangeSliderVisualUpdate {
                        fill_entity: data.fill_entity,
                        low_thumb: data.low_thumb_entity,
                        high_thumb: data.high_thumb_entity,
                        text_slot: data.text_slot,
                        precision: data.precision,
                        low_normalized: ((low_value - data.min) / range).clamp(0.0, 1.0),
                        high_normalized: ((high_value - data.min) / range).clamp(0.0, 1.0),
                        low_value,
                        high_value,
                        thumb_half: data.thumb_half_size,
                    },
                );
            }
        }

        range_sliders.for_entity(entity, |(widget_data,)| {
            widget_data.low_value = low_value;
            widget_data.high_value = high_value;
            widget_data.changed = changed;
            widget_data.active_thumb = active_thumb;
        });

        if changed {
            frame
                .events
                .push(crate::resources::UiEvent::RangeSliderChanged {
                    entity,
                    low: low_value,
                    high: high_value,
                });
        }
    }
}
