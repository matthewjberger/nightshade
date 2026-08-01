use nalgebra_glm;
use nalgebra_glm::Vec2;
use nightshade::ecs::system_param::{Query, Res};

pub type UiNodes<'world, 'item> = Query<'world, (&'item mut crate::components::UiLayoutNode,)>;

pub type UiScrollAreas<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiScrollAreaData,)>;

pub type UiInteractions<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiNodeInteraction,)>;

pub type UiParents<'world, 'item> = Query<'world, (&'item nightshade::ecs::ChildOf,)>;

pub type UiVirtualLists<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiVirtualListData,)>;

pub type UiContents<'world, 'item> = Query<'world, (&'item mut crate::components::UiNodeContent,)>;

nightshade::ecs::query_bundle! {
    pub struct ScrollSlots<'world, 'item> {
        pub nodes: UiNodes<'world, 'item>,
        pub areas: UiScrollAreas<'world, 'item>,
        pub interactions: UiInteractions<'world, 'item>,
        pub parents: UiParents<'world, 'item>,
        pub lists: UiVirtualLists<'world, 'item>,
        pub contents: UiContents<'world, 'item>,
    }
}

pub(super) struct ScrollFrameInputs {
    pub entity: nightshade::ecs::Entity,
    pub content_entity: nightshade::ecs::Entity,
    pub thumb_entity: nightshade::ecs::Entity,
    pub track_entity: nightshade::ecs::Entity,
    pub scroll_offset: f32,
    pub thumb_dragging: bool,
    pub thumb_drag_start_offset: f32,
    pub snap_interval: Option<f32>,
    pub scroll_delta: Vec2,
    pub mouse_position: Vec2,
    pub dpi_scale: f32,
    pub delta_time: f32,
    pub scroll_under_cursor: bool,
}

pub(super) struct ScrollFrameResult {
    pub scroll_offset: f32,
    pub thumb_dragging: bool,
    pub thumb_drag_start_offset: f32,
    pub content_height: f32,
    pub visible_height: f32,
}

pub(super) fn is_innermost_scroll_for(
    areas: &mut UiScrollAreas<'_, '_>,
    parents: &mut UiParents<'_, '_>,
    hovered: nightshade::ecs::Entity,
    scroll_entity: nightshade::ecs::Entity,
) -> bool {
    let mut current = hovered;
    loop {
        let mut is_area = false;
        areas.for_entity(current, |_| is_area = true);
        if is_area {
            return current == scroll_entity;
        }
        let mut parent = None;
        parents.for_entity(current, |(child_of,)| parent = Some(child_of.0));
        match parent {
            Some(parent) => current = parent,
            None => return false,
        }
    }
}

pub(super) fn is_within(
    parents: &mut UiParents<'_, '_>,
    entity: nightshade::ecs::Entity,
    ancestor: nightshade::ecs::Entity,
) -> bool {
    let mut current = entity;
    loop {
        if current == ancestor {
            return true;
        }
        let mut parent = None;
        parents.for_entity(current, |(child_of,)| parent = Some(child_of.0));
        match parent {
            Some(parent) => current = parent,
            None => return false,
        }
    }
}

pub(super) fn apply_scroll_frame(
    nodes: &mut UiNodes<'_, '_>,
    interactions: &mut UiInteractions<'_, '_>,
    children_of_content: &[nightshade::ecs::Entity],
    inputs: ScrollFrameInputs,
) -> ScrollFrameResult {
    let mut visible_height = 0.0;
    nodes.for_entity(inputs.entity, |(node,)| {
        visible_height = node.computed_rect.height()
    });

    let mut content_flow = None;
    nodes.for_entity(inputs.content_entity, |(node,)| {
        content_flow = node.flow_layout
    });

    let mut total_content_height = 0.0f32;
    let mut visible_count = 0usize;
    for child in children_of_content {
        nodes.for_entity(*child, |(node,)| {
            if node.visible {
                total_content_height += node.computed_rect.height();
                visible_count += 1;
            }
        });
    }

    if let Some(flow) = content_flow {
        if visible_count > 1 {
            total_content_height += flow.spacing * inputs.dpi_scale * (visible_count - 1) as f32;
        }
        total_content_height += flow.padding * inputs.dpi_scale * 2.0;
    }

    let max_scroll = (total_content_height - visible_height).max(0.0) / inputs.dpi_scale;
    let mut scroll_offset = inputs.scroll_offset;
    let mut thumb_dragging = inputs.thumb_dragging;
    let mut thumb_drag_start_offset = inputs.thumb_drag_start_offset;

    if inputs.scroll_under_cursor && inputs.scroll_delta.y.abs() > 0.0 {
        scroll_offset -= inputs.scroll_delta.y * 40.0;
        scroll_offset = scroll_offset.clamp(0.0, max_scroll);
    }

    let mut thumb_pressed = false;
    let mut thumb_drag_start = None;
    interactions.for_entity(inputs.thumb_entity, |(interaction,)| {
        thumb_pressed = interaction.pressed;
        thumb_drag_start = interaction.drag_start;
    });

    if thumb_pressed && !thumb_dragging {
        thumb_dragging = true;
        thumb_drag_start_offset = scroll_offset;
    }

    if thumb_dragging
        && thumb_pressed
        && let Some(drag_start) = thumb_drag_start
    {
        let mut track_rect = None;
        nodes.for_entity(inputs.track_entity, |(node,)| {
            track_rect = Some(node.computed_rect)
        });
        if let Some(track) = track_rect {
            let track_height = track.height();
            let thumb_ratio = if total_content_height > 0.0 {
                visible_height / total_content_height
            } else {
                1.0
            };
            let scrollable_track = track_height * (1.0 - thumb_ratio);
            if scrollable_track > 0.0 {
                let mouse_delta = inputs.mouse_position.y - drag_start.y;
                let scroll_per_pixel = max_scroll / scrollable_track;
                scroll_offset = (thumb_drag_start_offset + mouse_delta * scroll_per_pixel)
                    .clamp(0.0, max_scroll);
            }
        }
    }

    if !thumb_pressed {
        thumb_dragging = false;
    }

    if let Some(snap) = inputs.snap_interval
        && snap > 0.0
        && !thumb_dragging
        && inputs.scroll_delta.y.abs() < f32::EPSILON
    {
        let target = ((scroll_offset / snap).round() * snap).clamp(0.0, max_scroll);
        let difference = target - scroll_offset;
        if difference.abs() > 0.1 {
            scroll_offset += difference * (10.0 * inputs.delta_time).min(1.0);
        } else {
            scroll_offset = target;
        }
    }

    nodes.for_entity(inputs.content_entity, |(content_node,)| {
        content_node.scroll_offset = Vec2::new(0.0, scroll_offset)
    });

    let show_scrollbar = total_content_height > visible_height;
    nodes.for_entity(inputs.track_entity, |(track_node,)| {
        track_node.visible = show_scrollbar
    });

    if show_scrollbar {
        let thumb_ratio = (visible_height / total_content_height).clamp(0.05, 1.0);
        let mut track_rect = None;
        nodes.for_entity(inputs.track_entity, |(node,)| {
            track_rect = Some(node.computed_rect)
        });
        if let Some(track) = track_rect {
            let track_height = track.height();
            let thumb_height_physical = (track_height * thumb_ratio).max(20.0 * inputs.dpi_scale);
            let scrollable_track = track_height - thumb_height_physical;
            let scroll_ratio = if max_scroll > 0.0 {
                scroll_offset / max_scroll
            } else {
                0.0
            };
            let thumb_y = scroll_ratio * scrollable_track / inputs.dpi_scale;
            let thumb_height = thumb_height_physical / inputs.dpi_scale;

            nodes.for_entity(inputs.thumb_entity, |(thumb_node,)| {
                if let Some(crate::components::layout_types::UiLayoutType::Window(window)) =
                    thumb_node.base_layout.as_mut()
                {
                    window.position = crate::data::units::Ab(Vec2::new(0.0, thumb_y)).into();
                    window.size = crate::data::units::Ab(Vec2::new(8.0, thumb_height)).into();
                }
            });
        }
    }

    ScrollFrameResult {
        scroll_offset,
        thumb_dragging,
        thumb_drag_start_offset,
        content_height: total_content_height,
        visible_height,
    }
}

pub fn ui_scroll_area_system(
    timing: Res<crate::resources::RetainedUiTiming>,
    interaction: Res<crate::resources::RetainedUiInteraction>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    input: Res<nightshade_platform::contract::input::Input>,
    window: Res<nightshade_platform::contract::window::Window>,
    hierarchy: Res<nightshade::ecs::HierarchyIndex>,
    mut slots: ScrollSlots<'_, '_>,
) {
    let dpi_scale = window.cached_scale_factor;
    let mouse_position = input.mouse.position;

    let mut pending = Vec::new();
    slots.areas.for_each_mut(|entity, (data,)| {
        pending.push((entity, data.clone()));
    });

    for (entity, data) in pending {
        let children: Vec<nightshade::ecs::Entity> =
            hierarchy.children(data.content_entity).to_vec();
        let scroll_under_cursor = interaction.hovered_entity.is_some_and(|hovered| {
            is_innermost_scroll_for(&mut slots.areas, &mut slots.parents, hovered, entity)
        });
        let result = apply_scroll_frame(
            &mut slots.nodes,
            &mut slots.interactions,
            &children,
            ScrollFrameInputs {
                entity,
                content_entity: data.content_entity,
                thumb_entity: data.thumb_entity,
                track_entity: data.track_entity,
                scroll_offset: data.scroll_offset,
                thumb_dragging: data.thumb_dragging,
                thumb_drag_start_offset: data.thumb_drag_start_offset,
                snap_interval: data.snap_interval,
                scroll_delta: mirror.scroll_delta,
                mouse_position,
                dpi_scale,
                delta_time: timing.delta_time,
                scroll_under_cursor,
            },
        );

        slots.areas.for_entity(entity, |(widget_data,)| {
            widget_data.scroll_offset = result.scroll_offset;
            widget_data.content_height = result.content_height;
            widget_data.visible_height = result.visible_height;
            widget_data.thumb_dragging = result.thumb_dragging;
            widget_data.thumb_drag_start_offset = result.thumb_drag_start_offset;
        });
    }
}

pub fn ui_virtual_list_system(
    theme: Res<crate::resources::ThemeState>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    interaction: Res<crate::resources::RetainedUiInteraction>,
    input: Res<nightshade_platform::contract::input::Input>,
    window: Res<nightshade_platform::contract::window::Window>,
    mut frame: nightshade::ecs::system_param::ResMut<crate::resources::RetainedUiFrame>,
    mut slots: ScrollSlots<'_, '_>,
) {
    let dpi_scale = window.cached_scale_factor;
    let accent_color = theme.active_theme().accent_color;
    let screen_position = input.mouse.position;
    let focused_entity = interaction.focused_entity;

    let mut pending = Vec::new();
    slots
        .lists
        .for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        let mut scroll_offset = 0.0;
        slots
            .areas
            .for_entity(data.scroll_entity, |(scroll_data,)| {
                scroll_offset = scroll_data.scroll_offset
            });

        let pool = super::pool_window::compute(
            scroll_offset,
            data.item_height,
            data.total_items,
            data.pool_size,
        );
        let visible_start = pool.visible_start;

        slots.nodes.for_entity(data.top_spacer, |(node,)| {
            node.flow_child_size = Some(
                crate::data::units::Rl(Vec2::new(100.0, 0.0))
                    + crate::data::units::Ab(Vec2::new(0.0, pool.top_height)),
            )
        });
        slots.nodes.for_entity(data.bottom_spacer, |(node,)| {
            node.flow_child_size = Some(
                crate::data::units::Rl(Vec2::new(100.0, 0.0))
                    + crate::data::units::Ab(Vec2::new(0.0, pool.bottom_height)),
            )
        });

        let mut selection = data.selection;
        let mut selection_changed = false;

        for (pool_index, item) in data.pool_items.iter().enumerate() {
            let item_index = visible_start + pool_index;
            let is_visible = item_index < data.total_items;

            slots
                .nodes
                .for_entity(item.container_entity, |(node,)| node.visible = is_visible);
            if !is_visible {
                continue;
            }

            let mut clicked = false;
            let mut right_clicked = false;
            slots
                .interactions
                .for_entity(item.container_entity, |(node_interaction,)| {
                    clicked = node_interaction.clicked;
                    right_clicked = node_interaction.right_clicked;
                });
            if clicked {
                selection = Some(item_index);
                selection_changed = true;
                frame
                    .events
                    .push(crate::resources::UiEvent::VirtualListItemClicked { entity, item_index });
            }
            if right_clicked {
                frame
                    .events
                    .push(crate::resources::UiEvent::VirtualListItemRightClicked {
                        entity,
                        item_index,
                        screen_position,
                    });
            }

            let is_selected = selection == Some(item_index);
            slots
                .contents
                .for_entity(item.container_entity, |(content,)| {
                    if let crate::components::UiNodeContent::Rect {
                        border_width,
                        border_color,
                        ..
                    } = content
                    {
                        if is_selected {
                            *border_width = 1.0 / dpi_scale;
                            *border_color = accent_color;
                        } else {
                            *border_width = 0.0;
                        }
                    }
                });
        }

        let is_focused = focused_entity == Some(entity)
            || data
                .pool_items
                .iter()
                .any(|item| focused_entity == Some(item.container_entity));
        if is_focused && data.total_items > 0 {
            for &(key, pressed) in &mirror.frame_keys {
                if !pressed {
                    continue;
                }
                match key {
                    nightshade_platform::KeyCode::ArrowUp => {
                        let current = selection.unwrap_or(0);
                        if current > 0 {
                            selection = Some(current - 1);
                            selection_changed = true;
                        }
                    }
                    nightshade_platform::KeyCode::ArrowDown => {
                        let current = selection.map_or(0, |selected| selected + 1);
                        if current < data.total_items {
                            selection = Some(current);
                            selection_changed = true;
                        }
                    }
                    nightshade_platform::KeyCode::Home => {
                        selection = Some(0);
                        selection_changed = true;
                    }
                    nightshade_platform::KeyCode::End => {
                        selection = Some(data.total_items - 1);
                        selection_changed = true;
                    }
                    _ => {}
                }
            }
        }

        slots.lists.for_entity(entity, |(widget_data,)| {
            widget_data.visible_start = visible_start;
            widget_data.selection = selection;
            widget_data.selection_changed = selection_changed;
        });
    }
}
