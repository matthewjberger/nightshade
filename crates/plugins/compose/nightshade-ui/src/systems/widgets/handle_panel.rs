use crate::data::state::UiStateTrait as _;
use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};
use nightshade::ecs::system_param::{Commands, Query, Res, ResMut};

use super::handle_navigation::UiColors;
use super::handle_scroll::{UiInteractions, UiNodes};
use super::handle_slider::{node_rect, row_interaction};
use crate::components::UiPanelKind;

use crate::prelude::*;

pub type UiPanels<'world, 'item> = Query<'world, (&'item mut crate::components::UiPanelData,)>;

pub type UiColorWheels<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiColorWheelData,)>;

pub type UiColorPickers<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiColorPickerData,)>;

pub type UiSliderValues<'world, 'item> = Query<'world, (&'item crate::components::UiSliderData,)>;

pub type UiEffects<'world, 'item> = Query<'world, (&'item mut crate::components::UiNodeEffect,)>;

nightshade::ecs::query_bundle! {
    pub struct PanelSlots<'world, 'item> {
        pub panels: UiPanels<'world, 'item>,
        pub wheels: UiColorWheels<'world, 'item>,
        pub pickers: UiColorPickers<'world, 'item>,
        pub nodes: UiNodes<'world, 'item>,
        pub interactions: UiInteractions<'world, 'item>,
        pub sliders: UiSliderValues<'world, 'item>,
        pub colors: UiColors<'world, 'item>,
        pub effects: UiEffects<'world, 'item>,
    }
}

struct PanelContext {
    mouse_position: Vec2,
    mouse_just_pressed: bool,
    mouse_just_released: bool,
    mouse_down: bool,
    dpi_scale: f32,
    viewport_size: Vec2,
    mouse_occluded: bool,
}

fn handle_panel(
    window: &mut nightshade_platform::contract::window::Window,
    overlays: &mut crate::resources::RetainedUiOverlays,
    text_state: &mut nightshade_text::contract::resources::TextState,
    panels: &mut UiPanels<'_, '_>,
    nodes: &mut UiNodes<'_, '_>,
    interactions: &mut UiInteractions<'_, '_>,
    entity: nightshade::ecs::Entity,
    data: &crate::components::UiPanelData,
    context: &PanelContext,
) {
    use crate::components::ResizeEdge;
    use crate::data::units::Ab;

    let mouse_position = context.mouse_position;
    let mouse_just_pressed = context.mouse_just_pressed;
    let mouse_just_released = context.mouse_just_released;
    let mouse_down = context.mouse_down;
    let dpi_scale = context.dpi_scale;
    let viewport_size = context.viewport_size;

    let header_interaction = row_interaction(interactions, data.header_entity);

    let mut drag_offset = data.drag_offset;
    let mut resize_edge = data.resize_edge;
    let mut resize_start_rect = data.resize_start_rect;
    let mut resize_start_mouse = data.resize_start_mouse;
    let mut panel_kind = data.panel_kind;
    let mut undocked_rect = data.undocked_rect;
    let mut default_dock_size = data.default_dock_size;
    let resizable = data.resizable;

    if let Some(collapse_entity) = data.collapse_button_entity
        && row_interaction(interactions, collapse_entity).clicked
    {
        if panel_kind == UiPanelKind::Floating {
            nodes.for_entity(entity, |(node,)| node.visible = false);
        } else {
            let new_collapsed = !data.collapsed;
            nodes.for_entity(data.content_entity, |(node,)| node.visible = !new_collapsed);
            if let Some(text_slot) = data.collapse_button_text_slot {
                text_state
                    .cache
                    .set_text(text_slot, if new_collapsed { "+" } else { "-" });
            }
            panels.for_entity(entity, |(widget_data,)| {
                widget_data.collapsed = new_collapsed
            });
        }
    }

    if header_interaction.dragging && panel_kind == UiPanelKind::Floating && resize_edge.is_none() {
        if drag_offset.is_none()
            && let Some(rect) = node_rect(nodes, entity)
            && let Some(drag_start) = header_interaction.drag_start
        {
            drag_offset = Some(drag_start - rect.min);
        }

        if let Some(offset) = drag_offset {
            nodes.for_entity(entity, |(node,)| {
                let computed_min = node.computed_rect.min;
                if let Some(crate::components::layout_types::UiLayoutType::Window(panel_window)) =
                    node.base_layout.as_mut()
                {
                    let current_ab = panel_window
                        .position
                        .absolute
                        .unwrap_or(Vec2::new(0.0, 0.0));
                    let parent_min = computed_min - current_ab * dpi_scale;
                    let new_position = (mouse_position - offset - parent_min) / dpi_scale;
                    panel_window.position =
                        Ab(Vec2::new(new_position.x.round(), new_position.y.round())).into();
                }
            });
        }

        overlays.dock_indicator_active = true;
        overlays.dock_indicator_panel = Some(entity);
    }

    if header_interaction.dragging
        && panel_kind != UiPanelKind::Floating
        && !data.pinned
        && resize_edge.is_none()
        && let Some(drag_start) = header_interaction.drag_start
    {
        let drag_distance = (mouse_position - drag_start).magnitude();
        if drag_distance > 50.0 {
            panel_kind = UiPanelKind::Floating;

            let restored_size = undocked_rect
                .map(|rect| rect.size())
                .unwrap_or(Vec2::new(300.0, 400.0));
            let new_position = mouse_position - Vec2::new(restored_size.x * 0.5, 14.0);

            nodes.for_entity(entity, |(node,)| {
                node.layer = Some(crate::components::layer::UiLayer::FloatingPanels);
                if let Some(crate::components::layout_types::UiLayoutType::Window(panel_window)) =
                    node.base_layout.as_mut()
                {
                    panel_window.position = Ab(new_position / dpi_scale).into();
                    panel_window.size = Ab(restored_size / dpi_scale).into();
                }
            });

            drag_offset = Some(Vec2::new(restored_size.x * 0.5, 14.0));
            overlays.dock_indicator_active = true;
            overlays.dock_indicator_panel = Some(entity);
        }
    }

    if mouse_just_released
        && overlays.dock_indicator_panel == Some(entity)
        && panel_kind == UiPanelKind::Floating
    {
        let indicator_size = 40.0;
        let indicator_spacing = 4.0;
        let center = viewport_size * 0.5;
        let dock_buttons = [
            (
                nightshade_paint::data::Rect::from_center_size(
                    Vec2::new(center.x, center.y - indicator_size - indicator_spacing),
                    Vec2::new(indicator_size, indicator_size),
                ),
                UiPanelKind::DockedTop,
            ),
            (
                nightshade_paint::data::Rect::from_center_size(
                    Vec2::new(center.x, center.y + indicator_size + indicator_spacing),
                    Vec2::new(indicator_size, indicator_size),
                ),
                UiPanelKind::DockedBottom,
            ),
            (
                nightshade_paint::data::Rect::from_center_size(
                    Vec2::new(center.x - indicator_size - indicator_spacing, center.y),
                    Vec2::new(indicator_size, indicator_size),
                ),
                UiPanelKind::DockedLeft,
            ),
            (
                nightshade_paint::data::Rect::from_center_size(
                    Vec2::new(center.x + indicator_size + indicator_spacing, center.y),
                    Vec2::new(indicator_size, indicator_size),
                ),
                UiPanelKind::DockedRight,
            ),
        ];

        let mut target_dock = None;
        for (btn_rect, dock_kind) in &dock_buttons {
            if btn_rect.contains(mouse_position) {
                target_dock = Some(*dock_kind);
                break;
            }
        }
        if target_dock.is_none() {
            let edge_threshold = 80.0;
            target_dock = if mouse_position.y < edge_threshold {
                Some(UiPanelKind::DockedTop)
            } else if mouse_position.y > viewport_size.y - edge_threshold {
                Some(UiPanelKind::DockedBottom)
            } else if mouse_position.x < edge_threshold {
                Some(UiPanelKind::DockedLeft)
            } else if mouse_position.x > viewport_size.x - edge_threshold {
                Some(UiPanelKind::DockedRight)
            } else {
                None
            };
        }

        if let Some(new_kind) = target_dock {
            if let Some(rect) = node_rect(nodes, entity) {
                undocked_rect = Some(rect);
            }
            panel_kind = new_kind;
            default_dock_size = match new_kind {
                UiPanelKind::DockedLeft | UiPanelKind::DockedRight => {
                    undocked_rect.map(|rect| rect.width()).unwrap_or(300.0) / dpi_scale
                }
                UiPanelKind::DockedTop | UiPanelKind::DockedBottom => {
                    undocked_rect.map(|rect| rect.height()).unwrap_or(300.0) / dpi_scale
                }
                UiPanelKind::Floating => 300.0,
            };

            nodes.for_entity(entity, |(node,)| {
                node.layer = Some(crate::components::layer::UiLayer::DockedPanels)
            });
        }

        overlays.dock_indicator_active = false;
        overlays.dock_indicator_panel = None;
    }

    if !header_interaction.pressed && !header_interaction.dragging {
        drag_offset = None;
        if overlays.dock_indicator_panel == Some(entity) {
            overlays.dock_indicator_active = false;
            overlays.dock_indicator_panel = None;
        }
    }

    if !context.mouse_occluded
        && resizable
        && let Some(rect) = node_rect(nodes, entity)
    {
        let edge_margin = 6.0 * dpi_scale;
        let near_left = (mouse_position.x - rect.min.x).abs() < edge_margin;
        let near_right = (mouse_position.x - rect.max.x).abs() < edge_margin;
        let near_top = (mouse_position.y - rect.min.y).abs() < edge_margin;
        let near_bottom = (mouse_position.y - rect.max.y).abs() < edge_margin;
        let inside_x = mouse_position.x >= rect.min.x - edge_margin
            && mouse_position.x <= rect.max.x + edge_margin;
        let inside_y = mouse_position.y >= rect.min.y - edge_margin
            && mouse_position.y <= rect.max.y + edge_margin;

        let detected_edge = match panel_kind {
            UiPanelKind::DockedLeft => {
                if near_right && inside_y {
                    Some(ResizeEdge::Right)
                } else {
                    None
                }
            }
            UiPanelKind::DockedRight => {
                if near_left && inside_y {
                    Some(ResizeEdge::Left)
                } else {
                    None
                }
            }
            UiPanelKind::DockedTop => {
                if near_bottom && inside_x {
                    Some(ResizeEdge::Bottom)
                } else {
                    None
                }
            }
            UiPanelKind::DockedBottom => {
                if near_top && inside_x {
                    Some(ResizeEdge::Top)
                } else {
                    None
                }
            }
            UiPanelKind::Floating => {
                if near_top && near_left && inside_x && inside_y {
                    Some(ResizeEdge::TopLeft)
                } else if near_top && near_right && inside_x && inside_y {
                    Some(ResizeEdge::TopRight)
                } else if near_bottom && near_left && inside_x && inside_y {
                    Some(ResizeEdge::BottomLeft)
                } else if near_bottom && near_right && inside_x && inside_y {
                    Some(ResizeEdge::BottomRight)
                } else if near_left && inside_y {
                    Some(ResizeEdge::Left)
                } else if near_right && inside_y {
                    Some(ResizeEdge::Right)
                } else if near_top && inside_x {
                    Some(ResizeEdge::Top)
                } else if near_bottom && inside_x {
                    Some(ResizeEdge::Bottom)
                } else {
                    None
                }
            }
        };

        if let Some(edge) = detected_edge {
            window.requested_cursor = Some(match edge {
                ResizeEdge::Left | ResizeEdge::Right => nightshade_platform::CursorIcon::EwResize,
                ResizeEdge::Top | ResizeEdge::Bottom => nightshade_platform::CursorIcon::NsResize,
                ResizeEdge::TopLeft | ResizeEdge::BottomRight => {
                    nightshade_platform::CursorIcon::NwseResize
                }
                ResizeEdge::TopRight | ResizeEdge::BottomLeft => {
                    nightshade_platform::CursorIcon::NeswResize
                }
            });
        }

        if !header_interaction.dragging && mouse_just_pressed && detected_edge.is_some() {
            resize_edge = detected_edge;
            resize_start_rect = Some(rect);
            resize_start_mouse = Some(mouse_position);
        }
    }

    if let Some(edge) = resize_edge {
        window.requested_cursor = Some(match edge {
            ResizeEdge::Left | ResizeEdge::Right => nightshade_platform::CursorIcon::EwResize,
            ResizeEdge::Top | ResizeEdge::Bottom => nightshade_platform::CursorIcon::NsResize,
            ResizeEdge::TopLeft | ResizeEdge::BottomRight => {
                nightshade_platform::CursorIcon::NwseResize
            }
            ResizeEdge::TopRight | ResizeEdge::BottomLeft => {
                nightshade_platform::CursorIcon::NeswResize
            }
        });
        if mouse_down {
            if let (Some(start_rect), Some(start_mouse)) = (resize_start_rect, resize_start_mouse) {
                let delta = mouse_position - start_mouse;
                let min_size = data.min_size * dpi_scale;
                let mut new_position = start_rect.min;
                let mut new_size = start_rect.size();

                match edge {
                    ResizeEdge::Right => {
                        new_size.x = (start_rect.width() + delta.x).max(min_size.x);
                    }
                    ResizeEdge::Bottom => {
                        new_size.y = (start_rect.height() + delta.y).max(min_size.y);
                    }
                    ResizeEdge::Left => {
                        let new_width = (start_rect.width() - delta.x).max(min_size.x);
                        new_position.x = start_rect.max.x - new_width;
                        new_size.x = new_width;
                    }
                    ResizeEdge::Top => {
                        let new_height = (start_rect.height() - delta.y).max(min_size.y);
                        new_position.y = start_rect.max.y - new_height;
                        new_size.y = new_height;
                    }
                    ResizeEdge::TopLeft => {
                        let new_width = (start_rect.width() - delta.x).max(min_size.x);
                        let new_height = (start_rect.height() - delta.y).max(min_size.y);
                        new_position.x = start_rect.max.x - new_width;
                        new_position.y = start_rect.max.y - new_height;
                        new_size.x = new_width;
                        new_size.y = new_height;
                    }
                    ResizeEdge::TopRight => {
                        let new_height = (start_rect.height() - delta.y).max(min_size.y);
                        new_position.y = start_rect.max.y - new_height;
                        new_size.x = (start_rect.width() + delta.x).max(min_size.x);
                        new_size.y = new_height;
                    }
                    ResizeEdge::BottomLeft => {
                        let new_width = (start_rect.width() - delta.x).max(min_size.x);
                        new_position.x = start_rect.max.x - new_width;
                        new_size.x = new_width;
                        new_size.y = (start_rect.height() + delta.y).max(min_size.y);
                    }
                    ResizeEdge::BottomRight => {
                        new_size.x = (start_rect.width() + delta.x).max(min_size.x);
                        new_size.y = (start_rect.height() + delta.y).max(min_size.y);
                    }
                }

                if panel_kind != UiPanelKind::Floating {
                    match panel_kind {
                        UiPanelKind::DockedLeft | UiPanelKind::DockedRight => {
                            default_dock_size = new_size.x / dpi_scale;
                        }
                        UiPanelKind::DockedTop | UiPanelKind::DockedBottom => {
                            default_dock_size = new_size.y / dpi_scale;
                        }
                        UiPanelKind::Floating => {}
                    }
                } else {
                    nodes.for_entity(entity, |(node,)| {
                        let computed_min = node.computed_rect.min;
                        if let Some(crate::components::layout_types::UiLayoutType::Window(
                            panel_window,
                        )) = node.base_layout.as_mut()
                        {
                            let current_ab = panel_window
                                .position
                                .absolute
                                .unwrap_or(Vec2::new(0.0, 0.0));
                            let parent_min = computed_min - current_ab * dpi_scale;
                            let logical_position = (new_position - parent_min) / dpi_scale;
                            let logical_size = new_size / dpi_scale;
                            panel_window.position = Ab(Vec2::new(
                                logical_position.x.round(),
                                logical_position.y.round(),
                            ))
                            .into();
                            panel_window.size =
                                Ab(Vec2::new(logical_size.x.round(), logical_size.y.round()))
                                    .into();
                        }
                    });
                }
            }
        } else {
            resize_edge = None;
            resize_start_rect = None;
            resize_start_mouse = None;
        }
    }

    panels.for_entity(entity, |(widget_data,)| {
        widget_data.drag_offset = drag_offset;
        widget_data.resize_edge = resize_edge;
        widget_data.resize_start_rect = resize_start_rect;
        widget_data.resize_start_mouse = resize_start_mouse;
        widget_data.panel_kind = panel_kind;
        widget_data.undocked_rect = undocked_rect;
        widget_data.default_dock_size = default_dock_size;
    });
}

pub fn ui_panel_system(
    input: Res<nightshade_platform::contract::input::Input>,
    mut window: ResMut<nightshade_platform::contract::window::Window>,
    mut overlays: ResMut<crate::resources::RetainedUiOverlays>,
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    mut slots: PanelSlots<'_, '_>,
) {
    let state = input.mouse.state;
    let mouse_position = input.mouse.position;
    let dpi_scale = window.cached_scale_factor;
    let viewport_size = window
        .cached_viewport_size
        .map(|(width, height)| Vec2::new(width as f32, height as f32))
        .unwrap_or(Vec2::new(800.0, 600.0));

    let mut pending = Vec::new();
    slots
        .panels
        .for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    let mut panel_rects = Vec::new();
    for (entity, _) in &pending {
        slots.nodes.for_entity(*entity, |(node,)| {
            if node.visible {
                panel_rects.push((*entity, node.computed_rect, node.computed_depth));
            }
        });
    }

    for (entity, data) in pending {
        let mut panel_depth = 0.0;
        slots
            .nodes
            .for_entity(entity, |(node,)| panel_depth = node.computed_depth);
        let mouse_occluded = panel_rects
            .iter()
            .any(|(other_entity, other_rect, other_depth)| {
                *other_entity != entity
                    && *other_depth > panel_depth
                    && other_rect.contains(mouse_position)
            });
        handle_panel(
            &mut window,
            &mut overlays,
            &mut text_state,
            &mut slots.panels,
            &mut slots.nodes,
            &mut slots.interactions,
            entity,
            &data,
            &PanelContext {
                mouse_position,
                mouse_just_pressed: state
                    .contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_PRESSED),
                mouse_just_released: state
                    .contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_RELEASED),
                mouse_down: state
                    .contains(nightshade_platform::contract::input::MouseState::LEFT_CLICKED),
                dpi_scale,
                viewport_size,
                mouse_occluded,
            },
        );
    }
}

fn place_color_dot(
    nodes: &mut UiNodes<'_, '_>,
    data: &crate::components::UiColorWheelData,
    dpi_scale: f32,
    hue_degrees: f32,
    saturation: f32,
) {
    const DOT_SIZE: f32 = 12.0;
    let Some(rect) = node_rect(nodes, data.wheel_entity) else {
        return;
    };
    let dpi = dpi_scale.max(0.0001);
    let (width, height) = (rect.width() / dpi, rect.height() / dpi);
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    let (dot_x, dot_y) = crate::data::color::color_wheel_dot_offset(
        width,
        height,
        DOT_SIZE,
        hue_degrees,
        saturation,
    );
    let position = Vec2::new(dot_x, dot_y);
    nodes.for_entity(data.dot_entity, |(node,)| {
        if let Some(crate::components::layout_types::UiLayoutType::Window(dot_window)) =
            node.base_layout.as_mut()
        {
            dot_window.position = crate::data::units::Ab(position).into();
        }
    });
}

fn handle_color_wheel(
    frame: &mut crate::resources::RetainedUiFrame,
    wheels: &mut UiColorWheels<'_, '_>,
    nodes: &mut UiNodes<'_, '_>,
    interactions: &mut UiInteractions<'_, '_>,
    sliders: &mut UiSliderValues<'_, '_>,
    colors: &mut UiColors<'_, '_>,
    effects: &mut UiEffects<'_, '_>,
    dpi_scale: f32,
    mouse_position: Vec2,
    entity: nightshade::ecs::Entity,
    data: &crate::components::UiColorWheelData,
) {
    let mut hue = data.hue;
    let mut saturation = data.saturation;
    let mut value = data.value;
    let mut changed_position = false;

    let wheel_interaction = row_interaction(interactions, data.wheel_entity);
    if (wheel_interaction.pressed || wheel_interaction.dragging)
        && let Some(rect) = node_rect(nodes, data.wheel_entity)
    {
        let (width, height) = (rect.width(), rect.height());
        if width > 0.0 && height > 0.0 {
            let (new_hue_degrees, new_saturation) = crate::data::color::color_wheel_hue_saturation(
                width,
                height,
                mouse_position.x - rect.min.x,
                mouse_position.y - rect.min.y,
            );
            let new_hue = (new_hue_degrees / 360.0).clamp(0.0, 1.0);
            if (new_saturation - saturation).abs() > f32::EPSILON
                || (new_hue - hue).abs() > f32::EPSILON
            {
                saturation = new_saturation;
                hue = new_hue;
                changed_position = true;
            }
        }
    }

    let mut alpha = data.alpha;
    let mut changed_value = false;
    let mut changed_alpha = false;

    sliders.for_entity(data.value_slider_entity, |(slider_data,)| {
        let slider_value = slider_data.value.clamp(0.0, 1.0);
        if slider_data.changed && (slider_value - value).abs() > f32::EPSILON {
            changed_value = true;
        }
        value = slider_value;
    });
    sliders.for_entity(data.alpha_slider_entity, |(slider_data,)| {
        if slider_data.changed && (slider_data.value - alpha).abs() > f32::EPSILON {
            changed_alpha = true;
        }
        alpha = slider_data.value;
    });

    let any_changed = changed_position || changed_value || changed_alpha;

    let hsva = crate::data::color::Hsva {
        hue: hue * 360.0,
        saturation,
        value,
        alpha,
    };
    let rgba = hsva.to_rgba();

    if changed_value {
        effects.for_entity(data.wheel_entity, |(effect,)| {
            let (kind, params) = crate::resources::theme::RectEffect::ColorWheel { value }.encode();
            effect.kind = kind;
            effect.params = params;
        });
    }

    if any_changed {
        colors.for_entity(data.swatch_entity, |(swatch_color,)| {
            swatch_color.colors[crate::data::state::UiBase::INDEX] = Some(rgba)
        });

        if changed_position {
            place_color_dot(nodes, data, dpi_scale, hue * 360.0, saturation);
        }
    }

    wheels.for_entity(entity, |(widget_data,)| {
        widget_data.hue = hue;
        widget_data.saturation = saturation;
        widget_data.value = value;
        widget_data.alpha = alpha;
        widget_data.color = rgba;
        widget_data.changed = any_changed;
    });
    if any_changed {
        frame
            .events
            .push(crate::resources::UiEvent::ColorPickerChanged {
                entity,
                color: rgba,
            });
    }
}

pub fn ui_color_wheel_system(
    input: Res<nightshade_platform::contract::input::Input>,
    window: Res<nightshade_platform::contract::window::Window>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut slots: PanelSlots<'_, '_>,
) {
    let dpi_scale = window.cached_scale_factor;
    let mouse_position = input.mouse.position;

    let mut pending = Vec::new();
    slots
        .wheels
        .for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        handle_color_wheel(
            &mut frame,
            &mut slots.wheels,
            &mut slots.nodes,
            &mut slots.interactions,
            &mut slots.sliders,
            &mut slots.colors,
            &mut slots.effects,
            dpi_scale,
            mouse_position,
            entity,
            &data,
        );
    }
}

fn keep_popup_on_screen(
    nodes: &mut UiNodes<'_, '_>,
    dirty: &mut crate::resources::RetainedUiDirty,
    viewport_size: Vec2,
    dpi_scale: f32,
    owner: nightshade::ecs::Entity,
    popup: nightshade::ecs::Entity,
) {
    let dpi_scale = dpi_scale.max(0.0001);
    let Some(owner_rect) = node_rect(nodes, owner) else {
        return;
    };
    let mut moved = false;
    nodes.for_entity(popup, |(node,)| {
        let Some(crate::components::layout_types::UiLayoutType::Window(popup_window)) =
            node.base_layout.as_mut()
        else {
            return;
        };
        let preferred = popup_window
            .position
            .absolute
            .unwrap_or(Vec2::new(0.0, 0.0));
        let Some(size) = popup_window.size.absolute else {
            return;
        };
        let (popup_width, popup_height) = (size.x * dpi_scale, size.y * dpi_scale);
        if popup_width <= 0.0 || popup_height <= 0.0 {
            return;
        }

        let wanted = owner_rect.min + preferred * dpi_scale;
        let overflow_x = (wanted.x + popup_width - viewport_size.x).max(0.0);
        let overflow_y = (wanted.y + popup_height - viewport_size.y).max(0.0);
        let corrected = Vec2::new(
            (wanted.x - overflow_x).max(0.0),
            (wanted.y - overflow_y).max(0.0),
        );

        popup_window.position =
            crate::data::units::Ab((corrected - owner_rect.min) / dpi_scale).into();
        moved = true;
    });
    if moved {
        crate::commands::widgets::world_interaction::ui_mark_layout_dirty(dirty);
    }
}

fn handle_color_picker(
    dirty: &mut crate::resources::RetainedUiDirty,
    frame: &mut crate::resources::RetainedUiFrame,
    commands: &mut Commands,
    pickers: &mut UiColorPickers<'_, '_>,
    wheels: &mut UiColorWheels<'_, '_>,
    nodes: &mut UiNodes<'_, '_>,
    interactions: &mut UiInteractions<'_, '_>,
    sliders: &mut UiSliderValues<'_, '_>,
    colors: &mut UiColors<'_, '_>,
    effects: &mut UiEffects<'_, '_>,
    pointer: PickerPointer,
    entity: nightshade::ecs::Entity,
    data: &crate::components::UiColorPickerData,
) {
    let mode = data.mode;
    let mut any_changed = false;

    let mut slider_values = [0.0_f32; 4];
    for (index, slider_entity) in data.slider_entities.iter().enumerate() {
        sliders.for_entity(*slider_entity, |(slider_data,)| {
            if slider_data.changed {
                any_changed = true;
            }
            slider_values[index] = slider_data.value;
        });
    }

    let mut color = match mode {
        crate::components::ColorPickerMode::Rgb => Vec4::new(
            slider_values[0],
            slider_values[1],
            slider_values[2],
            slider_values[3],
        ),
        crate::components::ColorPickerMode::Hsv => {
            let hsv = crate::data::color::Hsva {
                hue: slider_values[0],
                saturation: slider_values[1],
                value: slider_values[2],
                alpha: slider_values[3],
            };
            hsv.to_rgba()
        }
    };

    let mut popup_open = data.popup_open;
    let swatch_clicked = row_interaction(interactions, data.swatch_entity).clicked;
    if swatch_clicked {
        popup_open = !popup_open;
    }

    let mut wheel_changed = (false, color);
    wheels.for_entity(data.popup_wheel_entity, |(wheel,)| {
        wheel_changed = (wheel.changed, wheel.color)
    });
    if wheel_changed.0 {
        color = wheel_changed.1;
        any_changed = true;
        sync_color_picker_sliders(commands, data, color);
    }

    if popup_open && !data.popup_open {
        sync_color_wheel_to_color(
            commands,
            wheels,
            nodes,
            colors,
            effects,
            pointer.dpi_scale,
            data.popup_wheel_entity,
            color,
        );
        keep_popup_on_screen(
            nodes,
            dirty,
            pointer.viewport_size,
            pointer.dpi_scale,
            entity,
            data.popup_entity,
        );
    }

    nodes.for_entity(data.popup_entity, |(node,)| node.visible = popup_open);

    if popup_open && pointer.left_just_pressed && !swatch_clicked {
        let inside_popup =
            node_rect(nodes, data.popup_entity).is_some_and(|rect| rect.contains(pointer.position));
        if !inside_popup {
            popup_open = false;
            nodes.for_entity(data.popup_entity, |(node,)| node.visible = false);
        }
    }

    if any_changed {
        colors.for_entity(data.swatch_entity, |(swatch_color,)| {
            swatch_color.colors[crate::data::state::UiBase::INDEX] = Some(color)
        });
    }

    pickers.for_entity(entity, |(widget_data,)| {
        widget_data.color = color;
        widget_data.changed = any_changed;
        widget_data.popup_open = popup_open;
    });
    if any_changed {
        frame
            .events
            .push(crate::resources::UiEvent::ColorPickerChanged { entity, color });
    }
}

#[derive(Clone, Copy)]
struct PickerPointer {
    position: Vec2,
    left_just_pressed: bool,
    viewport_size: Vec2,
    dpi_scale: f32,
}

pub fn ui_color_picker_system(
    input: Res<nightshade_platform::contract::input::Input>,
    window: Res<nightshade_platform::contract::window::Window>,
    mut dirty: ResMut<crate::resources::RetainedUiDirty>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut commands: Commands,
    mut slots: PanelSlots<'_, '_>,
) {
    let pointer = PickerPointer {
        position: input.mouse.position,
        left_just_pressed: input
            .mouse
            .state
            .contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_PRESSED),
        viewport_size: window
            .cached_viewport_size
            .map(|(width, height)| Vec2::new(width as f32, height as f32))
            .unwrap_or(Vec2::new(800.0, 600.0)),
        dpi_scale: window.cached_scale_factor,
    };

    let mut pending = Vec::new();
    slots
        .pickers
        .for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        handle_color_picker(
            &mut dirty,
            &mut frame,
            &mut commands,
            &mut slots.pickers,
            &mut slots.wheels,
            &mut slots.nodes,
            &mut slots.interactions,
            &mut slots.sliders,
            &mut slots.colors,
            &mut slots.effects,
            pointer,
            entity,
            &data,
        );
    }
}

fn sync_color_picker_sliders(
    commands: &mut Commands,
    data: &crate::components::UiColorPickerData,
    color: Vec4,
) {
    let values = match data.mode {
        crate::components::ColorPickerMode::Rgb => [color.x, color.y, color.z, color.w],
        crate::components::ColorPickerMode::Hsv => {
            let hsv = crate::data::color::Hsva::from_rgba(color);
            [hsv.hue, hsv.saturation, hsv.value, color.w]
        }
    };
    for (slider_entity, value) in data.slider_entities.iter().zip(values) {
        ui_slider_set_value(commands, *slider_entity, value);
    }
}

fn sync_color_wheel_to_color(
    commands: &mut Commands,
    wheels: &mut UiColorWheels<'_, '_>,
    nodes: &mut UiNodes<'_, '_>,
    colors: &mut UiColors<'_, '_>,
    effects: &mut UiEffects<'_, '_>,
    dpi_scale: f32,
    wheel_entity: nightshade::ecs::Entity,
    color: Vec4,
) {
    let mut wheel_data = None;
    wheels.for_entity(wheel_entity, |(data,)| wheel_data = Some(data.clone()));
    let Some(wheel_data) = wheel_data else {
        return;
    };
    let hsv = crate::data::color::Hsva::from_rgba(color);
    let hue_normalized = (hsv.hue / 360.0).clamp(0.0, 1.0);
    let saturation = hsv.saturation.clamp(0.0, 1.0);
    let value = hsv.value.clamp(0.0, 1.0);
    let alpha = color.w.clamp(0.0, 1.0);

    ui_slider_set_value(commands, wheel_data.value_slider_entity, value);
    ui_slider_set_value(commands, wheel_data.alpha_slider_entity, alpha);

    colors.for_entity(wheel_data.swatch_entity, |(swatch_color,)| {
        swatch_color.colors[crate::data::state::UiBase::INDEX] = Some(color)
    });
    effects.for_entity(wheel_data.wheel_entity, |(effect,)| {
        let (kind, params) = crate::resources::theme::RectEffect::ColorWheel { value }.encode();
        effect.kind = kind;
        effect.params = params;
    });
    place_color_dot(nodes, &wheel_data, dpi_scale, hsv.hue, saturation);

    wheels.for_entity(wheel_entity, |(widget_data,)| {
        widget_data.color = color;
        widget_data.hue = hue_normalized;
        widget_data.saturation = saturation;
        widget_data.value = value;
        widget_data.alpha = alpha;
    });
}
