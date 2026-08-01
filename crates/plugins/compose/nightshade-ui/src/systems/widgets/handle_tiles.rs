use nalgebra_glm;
use nalgebra_glm::Vec2;
use nightshade::ecs::system_param::{Query, Res, ResMut};

use super::handle_scroll::{UiNodes, UiParents, is_within};

use crate::components::{
    DropPreview, DropZone, SplitDirection, TileId, TileNode, UiTileContainerData,
};

use crate::prelude::*;
fn compute_tile_rects(
    data: &mut UiTileContainerData,
    tile_id: TileId,
    available: nightshade_paint::data::Rect,
) {
    if tile_id.0 >= data.rects.len() {
        data.rects
            .resize(tile_id.0 + 1, nightshade_paint::data::Rect::default());
    }
    data.rects[tile_id.0] = available;

    let Some(node) = data.tiles[tile_id.0].clone() else {
        return;
    };

    let collapse_to_zero = available.width() <= 0.0 || available.height() <= 0.0;
    let zero_rect = nightshade_paint::data::Rect {
        min: Vec2::new(0.0, 0.0),
        max: Vec2::new(0.0, 0.0),
    };

    match node {
        TileNode::Pane { .. } => {}
        TileNode::Split { children, .. } if collapse_to_zero => {
            for child in children {
                compute_tile_rects(data, child, zero_rect);
            }
        }
        TileNode::Split {
            direction,
            ratio,
            children,
        } => {
            let splitter = data.splitter_width;
            match direction {
                SplitDirection::Horizontal => {
                    let total_width = available.width() - splitter;
                    let left_width = total_width * ratio;

                    let left_rect = nightshade_paint::data::Rect {
                        min: available.min,
                        max: Vec2::new(available.min.x + left_width, available.max.y),
                    };
                    let right_rect = nightshade_paint::data::Rect {
                        min: Vec2::new(available.min.x + left_width + splitter, available.min.y),
                        max: available.max,
                    };

                    compute_tile_rects(data, children[0], left_rect);
                    compute_tile_rects(data, children[1], right_rect);
                }
                SplitDirection::Vertical => {
                    let total_height = available.height() - splitter;
                    let top_height = total_height * ratio;

                    let top_rect = nightshade_paint::data::Rect {
                        min: available.min,
                        max: Vec2::new(available.max.x, available.min.y + top_height),
                    };
                    let bottom_rect = nightshade_paint::data::Rect {
                        min: Vec2::new(available.min.x, available.min.y + top_height + splitter),
                        max: available.max,
                    };

                    compute_tile_rects(data, children[0], top_rect);
                    compute_tile_rects(data, children[1], bottom_rect);
                }
            }
        }
        TileNode::Tabs { panes, active } => {
            let tab_bar_h = data.tab_bar_height;
            let content_rect = nightshade_paint::data::Rect {
                min: Vec2::new(available.min.x, available.min.y + tab_bar_h),
                max: available.max,
            };
            for (index, &pane_id) in panes.iter().enumerate() {
                if pane_id.0 >= data.rects.len() {
                    data.rects
                        .resize(pane_id.0 + 1, nightshade_paint::data::Rect::default());
                }
                if index == active {
                    compute_tile_rects(data, pane_id, content_rect);
                } else {
                    compute_tile_rects(data, pane_id, zero_rect);
                }
            }
        }
    }
}

fn tile_hit_test_splitter(data: &UiTileContainerData, mouse: Vec2) -> Option<TileId> {
    for (index, tile) in data.tiles.iter().enumerate() {
        if let Some(TileNode::Split {
            direction,
            children,
            ..
        }) = tile
        {
            let Some(child0_rect) = data.rects.get(children[0].0) else {
                continue;
            };
            let splitter_half = data.splitter_width * 0.5 + 2.0;
            let Some(parent_rect) = data.rects.get(index) else {
                continue;
            };

            let hit = match direction {
                SplitDirection::Horizontal => {
                    let boundary = child0_rect.max.x + data.splitter_width * 0.5;
                    (mouse.x - boundary).abs() < splitter_half
                        && mouse.y >= parent_rect.min.y
                        && mouse.y <= parent_rect.max.y
                }
                SplitDirection::Vertical => {
                    let boundary = child0_rect.max.y + data.splitter_width * 0.5;
                    (mouse.y - boundary).abs() < splitter_half
                        && mouse.x >= parent_rect.min.x
                        && mouse.x <= parent_rect.max.x
                }
            };

            if hit {
                return Some(TileId(index));
            }
        }
    }
    None
}

fn tile_hit_test_tab_bar(data: &UiTileContainerData, mouse: Vec2) -> Option<(TileId, usize)> {
    for (index, tile) in data.tiles.iter().enumerate() {
        if let Some(TileNode::Tabs { panes, .. }) = tile
            && !panes.is_empty()
        {
            let Some(rect) = data.rects.get(index) else {
                continue;
            };
            let tab_bar_rect = nightshade_paint::data::Rect {
                min: rect.min,
                max: Vec2::new(rect.max.x, rect.min.y + data.tab_bar_height),
            };

            if tab_bar_rect.contains(mouse) {
                let tab_width = tab_bar_rect.width() / panes.len() as f32;
                let relative_x = mouse.x - tab_bar_rect.min.x;
                let tab_index = (relative_x / tab_width) as usize;
                let tab_index = tab_index.min(panes.len() - 1);
                return Some((TileId(index), tab_index));
            }
        }
    }
    None
}

fn tile_hit_test_tab_close(
    data: &UiTileContainerData,
    tabs_id: TileId,
    tab_index: usize,
    mouse: Vec2,
) -> bool {
    let Some(rect) = data.rects.get(tabs_id.0) else {
        return false;
    };
    if let Some(TileNode::Tabs { panes, .. }) = data.get(tabs_id)
        && !panes.is_empty()
    {
        let close_w = data.tab_close_width();
        let tab_width = rect.width() / panes.len() as f32;
        let tab_x = rect.min.x + tab_index as f32 * tab_width;
        let close_region_left = tab_x + tab_width - close_w;
        let close_region_right = tab_x + tab_width - 1.0;
        let close_region_top = rect.min.y + 1.0;
        let close_region_bottom = rect.min.y + data.tab_bar_height - 1.0;
        mouse.x >= close_region_left
            && mouse.x <= close_region_right
            && mouse.y >= close_region_top
            && mouse.y <= close_region_bottom
    } else {
        false
    }
}

fn tile_hit_test_drop_zone(data: &UiTileContainerData, mouse: Vec2) -> Option<DropPreview> {
    for (index, tile) in data.tiles.iter().enumerate() {
        if !matches!(tile, Some(TileNode::Tabs { .. })) {
            continue;
        }

        let rect = &data.rects[index];
        if !rect.contains(mouse) {
            continue;
        }

        let width = rect.width();
        let height = rect.height();
        let relative = mouse - rect.min;

        let zone_fraction = 0.25;

        let zone = if relative.x < width * zone_fraction {
            DropZone::Left
        } else if relative.x > width * (1.0 - zone_fraction) {
            DropZone::Right
        } else if relative.y < height * zone_fraction {
            DropZone::Top
        } else if relative.y > height * (1.0 - zone_fraction) {
            DropZone::Bottom
        } else {
            DropZone::Center
        };

        return Some(DropPreview {
            target_tile: TileId(index),
            zone,
        });
    }
    None
}

pub type UiTiles<'world, 'item> = Query<'world, (&'item mut UiTileContainerData,)>;

struct TileContainerContext {
    mouse_position: Vec2,
    mouse_just_pressed: bool,
    mouse_just_released: bool,
    mouse_down: bool,
    hovered_entity: Option<nightshade::ecs::Entity>,
    dpi_scale: f32,
}

fn handle_tile_container(
    window: &mut nightshade_platform::contract::window::Window,
    frame: &mut crate::resources::RetainedUiFrame,
    dirty: &mut crate::resources::RetainedUiDirty,
    tiles: &mut UiTiles<'_, '_>,
    nodes: &mut UiNodes<'_, '_>,
    parents: &mut UiParents<'_, '_>,
    entity: nightshade::ecs::Entity,
    data: &UiTileContainerData,
    context: &TileContainerContext,
) {
    let mut container_rect = nightshade_paint::data::Rect::default();
    let mut laid_out = false;
    nodes.for_entity(entity, |(node,)| {
        laid_out =
            node.visible && (node.computed_rect.width() > 0.0 || node.computed_rect.height() > 0.0);
        container_rect = node.computed_rect;
    });
    if !laid_out {
        return;
    }

    let mouse_position = context.mouse_position;
    let pointer_blocked = context
        .hovered_entity
        .is_some_and(|hovered| !is_within(parents, hovered, entity));
    let mouse_just_pressed = context.mouse_just_pressed && !pointer_blocked;
    let mouse_just_released = context.mouse_just_released;
    let mouse_down = context.mouse_down;

    let dpi_scale = context.dpi_scale;

    let mut data = data.clone();
    let prev_rects = data.rects.clone();

    let root = data.root;
    compute_tile_rects(&mut data, root, container_rect);

    let total_panes = data
        .tiles
        .iter()
        .filter(|tile| matches!(tile, Some(TileNode::Pane { .. })))
        .count();

    if let Some((split_id, _start_ratio)) = data.dragging_splitter {
        if mouse_down {
            let parent_rect = data.rects[split_id.0];
            let splitter_w = data.splitter_width;
            if let Some(TileNode::Split {
                direction, ratio, ..
            }) = data.get_mut(split_id)
            {
                let new_ratio = match direction {
                    SplitDirection::Horizontal => {
                        let total = parent_rect.width() - splitter_w;
                        if total > 0.0 {
                            ((mouse_position.x - parent_rect.min.x) / total).clamp(0.1, 0.9)
                        } else {
                            *ratio
                        }
                    }
                    SplitDirection::Vertical => {
                        let total = parent_rect.height() - splitter_w;
                        if total > 0.0 {
                            ((mouse_position.y - parent_rect.min.y) / total).clamp(0.1, 0.9)
                        } else {
                            *ratio
                        }
                    }
                };
                *ratio = new_ratio;
            }

            let root = data.root;
            compute_tile_rects(&mut data, root, container_rect);
        }
        if mouse_just_released || !mouse_down {
            if mouse_just_released && let Some(TileNode::Split { ratio, .. }) = data.get(split_id) {
                frame
                    .events
                    .push(crate::resources::UiEvent::TileSplitterMoved {
                        container: entity,
                        split_id,
                        ratio: *ratio,
                    });
            }
            data.dragging_splitter = None;
        }
    } else if let Some((source_tabs_id, tab_index, start_position)) = data.pending_tab_drag {
        if mouse_down {
            let distance = nalgebra_glm::length(&(mouse_position - start_position));
            if distance > 8.0 {
                data.pending_tab_drag = None;
                data.dragging_tab = Some((source_tabs_id, tab_index, mouse_position));
                data.drop_preview = tile_hit_test_drop_zone(&data, mouse_position);
            }
        } else {
            data.pending_tab_drag = None;
        }
    } else if let Some((source_tabs_id, tab_index, _drag_offset)) = data.dragging_tab {
        if mouse_down {
            data.dragging_tab = Some((source_tabs_id, tab_index, mouse_position));
            data.drop_preview = tile_hit_test_drop_zone(&data, mouse_position);
        }
        if mouse_just_released || !mouse_down {
            let preview = data.drop_preview.take();
            data.dragging_tab = None;

            if mouse_just_released
                && let Some(preview) = preview
                && let Some(TileNode::Tabs { panes, .. }) = data.get(source_tabs_id).cloned()
                && tab_index < panes.len()
            {
                let pane_id = panes[tab_index];
                let target = preview.target_tile;

                match preview.zone {
                    DropZone::Center => {
                        if target == source_tabs_id {
                            if let Some((_, drop_index)) =
                                tile_hit_test_tab_bar(&data, mouse_position)
                                    .filter(|(tid, _)| *tid == source_tabs_id)
                                && let Some(TileNode::Tabs { panes, active }) =
                                    data.get_mut(source_tabs_id)
                                && tab_index < panes.len()
                                && drop_index < panes.len()
                                && drop_index != tab_index
                            {
                                let moved = panes.remove(tab_index);
                                panes.insert(drop_index, moved);
                                *active = drop_index;
                            }
                        } else {
                            if let Some(TileNode::Tabs {
                                panes: source_panes,
                                active: source_active,
                            }) = data.get_mut(source_tabs_id)
                            {
                                source_panes.retain(|id| *id != pane_id);
                                if *source_active >= source_panes.len() && !source_panes.is_empty()
                                {
                                    *source_active = source_panes.len() - 1;
                                }
                            }
                            if let Some(TileNode::Tabs {
                                panes: target_panes,
                                active: target_active,
                            }) = data.get_mut(target)
                            {
                                target_panes.push(pane_id);
                                *target_active = target_panes.len() - 1;
                            }
                            frame
                                .events
                                .push(crate::resources::UiEvent::TileTabActivated {
                                    container: entity,
                                    pane_id,
                                });
                            collapse_empty_tiles(&mut data, source_tabs_id);
                        }
                    }
                    zone => {
                        if target == source_tabs_id && panes.len() == 1 {
                        } else {
                            let (direction, swap) = match zone {
                                DropZone::Left => (SplitDirection::Horizontal, true),
                                DropZone::Right => (SplitDirection::Horizontal, false),
                                DropZone::Top => (SplitDirection::Vertical, true),
                                DropZone::Bottom => (SplitDirection::Vertical, false),
                                DropZone::Center => unreachable!(),
                            };

                            if let Some(TileNode::Tabs {
                                panes: source_panes,
                                active: source_active,
                            }) = data.get_mut(source_tabs_id)
                            {
                                source_panes.retain(|id| *id != pane_id);
                                if *source_active >= source_panes.len() && !source_panes.is_empty()
                                {
                                    *source_active = source_panes.len() - 1;
                                }
                            }

                            let new_tabs = TileNode::Tabs {
                                panes: vec![pane_id],
                                active: 0,
                            };
                            let new_tabs_id = data.alloc(new_tabs);

                            if let Some(old_node) = data.tiles[target.0].take() {
                                let old_id = data.alloc(old_node);

                                let children = if swap {
                                    [new_tabs_id, old_id]
                                } else {
                                    [old_id, new_tabs_id]
                                };
                                data.tiles[target.0] = Some(TileNode::Split {
                                    direction,
                                    ratio: 0.5,
                                    children,
                                });
                            }

                            collapse_empty_tiles(&mut data, source_tabs_id);
                        }
                    }
                }
            }
        }
    } else {
        if mouse_just_pressed && container_rect.contains(mouse_position) {
            if let Some(split_id) = tile_hit_test_splitter(&data, mouse_position) {
                let current_ratio = if let Some(TileNode::Split { ratio, .. }) = data.get(split_id)
                {
                    *ratio
                } else {
                    0.5
                };
                data.dragging_splitter = Some((split_id, current_ratio));
            } else if let Some((tabs_id, tab_index)) = tile_hit_test_tab_bar(&data, mouse_position)
            {
                let close_hit = total_panes >= 2
                    && tile_hit_test_tab_close(&data, tabs_id, tab_index, mouse_position);

                if close_hit {
                    if let Some(TileNode::Tabs { panes, .. }) = data.get(tabs_id)
                        && tab_index < panes.len()
                    {
                        let pane_id = panes[tab_index];

                        let title = if let Some(TileNode::Pane { title, .. }) = data.get(pane_id) {
                            title.clone()
                        } else {
                            String::new()
                        };

                        if let Some(TileNode::Pane { content_entity, .. }) = data.get(pane_id) {
                            nodes.for_entity(*content_entity, |(node,)| node.visible = false);
                        }

                        if let Some(TileNode::Tabs { panes, active }) = data.get_mut(tabs_id) {
                            panes.retain(|id| *id != pane_id);
                            if *active >= panes.len() && !panes.is_empty() {
                                *active = panes.len() - 1;
                            }
                        }
                        data.free(pane_id);
                        collapse_empty_tiles(&mut data, tabs_id);

                        frame.events.push(crate::resources::UiEvent::TileTabClosed {
                            container: entity,
                            pane_id,
                            title,
                        });
                    }
                } else if let Some(TileNode::Tabs { panes, active }) = data.get_mut(tabs_id)
                    && tab_index < panes.len()
                {
                    let pane_id = panes[tab_index];
                    *active = tab_index;
                    data.pending_tab_drag = Some((tabs_id, tab_index, mouse_position));
                    frame
                        .events
                        .push(crate::resources::UiEvent::TileTabActivated {
                            container: entity,
                            pane_id,
                        });
                }
            }
        }

        data.drop_preview = None;
    }

    let root = data.root;
    compute_tile_rects(&mut data, root, container_rect);

    let mut parent_of: std::collections::HashMap<TileId, TileId> = std::collections::HashMap::new();
    for (parent_index, tile) in data.tiles.iter().enumerate() {
        match tile {
            Some(TileNode::Tabs { panes, .. }) => {
                for pane in panes {
                    parent_of.insert(*pane, TileId(parent_index));
                }
            }
            Some(TileNode::Split { children, .. }) => {
                for child in children {
                    parent_of.insert(*child, TileId(parent_index));
                }
            }
            _ => {}
        }
    }

    for (index, tile) in data.tiles.iter().enumerate() {
        if let Some(TileNode::Pane { content_entity, .. }) = tile {
            let rect = data.rects.get(index).copied().unwrap_or_default();

            let mut is_active = true;
            let mut current = TileId(index);
            while let Some(parent) = parent_of.get(&current).copied() {
                if let Some(TileNode::Tabs { panes, active }) = data.get(parent)
                    && panes.get(*active) != Some(&current)
                {
                    is_active = false;
                    break;
                }
                current = parent;
            }

            nodes.for_entity(*content_entity, |(node,)| {
                node.visible = is_active;
                if let Some(crate::components::layout_types::UiLayoutType::Window(window)) =
                    &mut node.base_layout
                {
                    window.position = crate::data::units::Ab(Vec2::new(
                        (rect.min.x - container_rect.min.x) / dpi_scale,
                        (rect.min.y - container_rect.min.y) / dpi_scale,
                    ))
                    .into();
                    window.size = crate::data::units::Ab(Vec2::new(
                        rect.width() / dpi_scale,
                        rect.height() / dpi_scale,
                    ))
                    .into();
                }
            });
        }
    }

    if let Some((split_id, _)) = data.dragging_splitter {
        if let Some(TileNode::Split { direction, .. }) = data.get(split_id) {
            let cursor = match direction {
                SplitDirection::Horizontal => nightshade_platform::CursorIcon::ColResize,
                SplitDirection::Vertical => nightshade_platform::CursorIcon::RowResize,
            };
            window.requested_cursor = Some(cursor);
        }
    } else if container_rect.contains(mouse_position)
        && let Some(split_id) = tile_hit_test_splitter(&data, mouse_position)
        && let Some(TileNode::Split { direction, .. }) = data.get(split_id)
    {
        let cursor = match direction {
            SplitDirection::Horizontal => nightshade_platform::CursorIcon::ColResize,
            SplitDirection::Vertical => nightshade_platform::CursorIcon::RowResize,
        };
        window.requested_cursor = Some(cursor);
    }

    data.hovered_close = if total_panes >= 2
        && data.dragging_tab.is_none()
        && data.dragging_splitter.is_none()
        && container_rect.contains(mouse_position)
    {
        if let Some((tabs_id, tab_index)) = tile_hit_test_tab_bar(&data, mouse_position)
            && tile_hit_test_tab_close(&data, tabs_id, tab_index, mouse_position)
        {
            Some((tabs_id, tab_index))
        } else {
            None
        }
    } else {
        None
    };

    if data.rects != prev_rects {
        ui_mark_layout_dirty(dirty);
    }

    tiles.for_entity(entity, |(widget_data,)| *widget_data = data.clone());
}

pub fn ui_tile_container_system(
    interaction: Res<crate::resources::RetainedUiInteraction>,
    input: Res<nightshade_platform::contract::input::Input>,
    mut window: ResMut<nightshade_platform::contract::window::Window>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut dirty: ResMut<crate::resources::RetainedUiDirty>,
    mut tiles: UiTiles<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut parents: UiParents<'_, '_>,
) {
    let state = input.mouse.state;
    let context = TileContainerContext {
        mouse_position: input.mouse.position,
        mouse_just_pressed: state
            .contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_PRESSED),
        mouse_just_released: state
            .contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_RELEASED),
        mouse_down: state.contains(nightshade_platform::contract::input::MouseState::LEFT_CLICKED),
        hovered_entity: interaction.hovered_entity,
        dpi_scale: window.cached_scale_factor.max(0.0001),
    };

    let mut pending = Vec::new();
    tiles.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        handle_tile_container(
            &mut window,
            &mut frame,
            &mut dirty,
            &mut tiles,
            &mut nodes,
            &mut parents,
            entity,
            &data,
            &context,
        );
    }
}

fn collapse_empty_tiles(data: &mut UiTileContainerData, tabs_id: TileId) {
    let remaining = if let Some(TileNode::Tabs { panes, .. }) = data.get(tabs_id) {
        panes.len()
    } else {
        return;
    };

    if remaining > 0 {
        return;
    }

    if let Some((parent_split_id, child_index)) = data.find_parent_split(tabs_id) {
        let sibling_index = 1 - child_index;
        let sibling_id = if let Some(TileNode::Split { children, .. }) = data.get(parent_split_id) {
            children[sibling_index]
        } else {
            return;
        };

        let sibling_node = data.tiles[sibling_id.0].take();
        data.tiles[parent_split_id.0] = sibling_node;
        data.free(tabs_id);
        data.free(sibling_id);
    }
}
