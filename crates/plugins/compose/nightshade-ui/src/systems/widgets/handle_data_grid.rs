use crate::data::state::UiStateTrait as _;
use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};
use nightshade::ecs::system_param::{Commands, Query, Res, ResMut};
use nightshade_platform::KeyCode;

use super::handle_navigation::{UiColors, UiWeights};
use super::handle_scroll::{UiInteractions, UiNodes, UiScrollAreas};
use super::handle_slider::{node_rect, row_interaction};

use crate::prelude::*;

pub type UiDataGrids<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiDataGridData,)>;

pub type UiPropertyGrids<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiPropertyGridData,)>;

pub type UiCommandPalettes<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiCommandPaletteData,)>;

pub type UiEditNodes<'world, 'item> = Query<
    'world,
    (
        &'item mut crate::components::UiLayoutNode,
        Option<&'item mut crate::components::UiTextInputData>,
    ),
>;

struct DataGridContext<'a> {
    ctrl_held: bool,
    shift_held: bool,
    frame_keys: &'a [(KeyCode, bool)],
    mouse_position: Vec2,
    mouse_just_pressed: bool,
    mouse_down: bool,
    viewport_size: Vec2,
}

fn edit_node_rect(
    nodes: &mut UiEditNodes<'_, '_>,
    entity: nightshade::ecs::Entity,
) -> Option<nightshade_paint::data::Rect> {
    let mut rect = None;
    nodes.for_entity(entity, |(node, _)| rect = Some(node.computed_rect));
    rect
}

fn handle_data_grid(
    window: &mut nightshade_platform::contract::window::Window,
    interaction_state: &mut crate::resources::RetainedUiInteraction,
    text_state: &mut nightshade_text::contract::resources::TextState,
    commands: &mut Commands,
    grids: &mut UiDataGrids<'_, '_>,
    nodes: &mut UiEditNodes<'_, '_>,
    interactions: &mut UiInteractions<'_, '_>,
    scroll_areas: &mut UiScrollAreas<'_, '_>,
    weights: &mut UiWeights<'_, '_>,
    entity: nightshade::ecs::Entity,
    data: &crate::components::UiDataGridData,
    context: &DataGridContext<'_>,
) {
    grids.for_entity(entity, |(grid,)| {
        grid.sort_changed = false;
        grid.selection_changed = false;
    });

    let effective_row_count = data
        .filtered_indices
        .as_ref()
        .map_or(data.total_rows, |indices| indices.len());

    let mut clicked_row = false;
    for (pool_index, pool_row) in data.pool_rows.iter().enumerate() {
        if !row_interaction(interactions, pool_row.row_entity).clicked {
            continue;
        }
        clicked_row = true;

        let data_row = data.visible_start + pool_index;
        if data_row >= effective_row_count {
            continue;
        }

        grids.for_entity(entity, |(grid,)| {
            grid.focused = true;
            if context.ctrl_held {
                if grid.selected_rows.contains(&data_row) {
                    grid.selected_rows.remove(&data_row);
                } else {
                    grid.selected_rows.insert(data_row);
                }
                grid.selection_anchor = Some(data_row);
            } else if context.shift_held
                && let Some(anchor) = grid.selection_anchor
            {
                grid.selected_rows.clear();
                for row in anchor.min(data_row)..=anchor.max(data_row) {
                    grid.selected_rows.insert(row);
                }
            } else {
                grid.selected_rows.clear();
                grid.selected_rows.insert(data_row);
                grid.selection_anchor = Some(data_row);
            }
            grid.selection_changed = true;
        });
        break;
    }

    if context.mouse_just_pressed
        && !clicked_row
        && let Some(rect) = edit_node_rect(nodes, entity)
        && !rect.contains(context.mouse_position)
    {
        grids.for_entity(entity, |(grid,)| grid.focused = false);
    }

    for (column_index, header_entity) in data.header_entities.iter().enumerate() {
        if column_index < data.columns.len()
            && data.columns[column_index].sortable
            && row_interaction(interactions, *header_entity).clicked
        {
            grids.for_entity(entity, |(grid,)| {
                if grid.sort_column == Some(column_index) {
                    if grid.sort_ascending {
                        grid.sort_ascending = false;
                    } else {
                        grid.sort_column = None;
                    }
                } else {
                    grid.sort_column = Some(column_index);
                    grid.sort_ascending = true;
                }
                grid.sort_changed = true;
            });

            data_grid_update_sort_indicators(grids, text_state, entity, data);
            break;
        }
    }

    for (pool_index, pool_row) in data.pool_rows.iter().enumerate() {
        if !row_interaction(interactions, pool_row.row_entity).double_clicked {
            continue;
        }

        let visible_row = data.visible_start + pool_index;
        if visible_row >= effective_row_count {
            continue;
        }

        let mouse_x = context.mouse_position.x;
        let mut found_column = None;
        for (column_index, cell_entity) in pool_row.cell_entities.iter().enumerate() {
            if column_index < data.columns.len()
                && data.columns[column_index].editable
                && let Some(cell_rect) = edit_node_rect(nodes, *cell_entity)
                && mouse_x >= cell_rect.min.x
                && mouse_x <= cell_rect.max.x
            {
                found_column = Some(column_index);
                break;
            }
        }

        if let Some(column) = found_column {
            ui_data_grid_start_edit(
                grids,
                nodes,
                text_state,
                interaction_state,
                commands,
                entity,
                visible_row,
                column,
            );
        }
        break;
    }

    if let Some(editing_input) = data.editing_input_entity
        && data.editing_cell.is_some()
    {
        let focused = interaction_state.focused_entity == Some(editing_input);
        let mut commit = false;
        let mut cancel = false;

        for &(key, pressed) in context.frame_keys {
            if !pressed {
                continue;
            }
            match key {
                KeyCode::Enter => commit = true,
                KeyCode::Escape => cancel = true,
                _ => {}
            }
        }

        if !focused && !commit && !cancel {
            commit = true;
        }

        if commit || cancel {
            ui_data_grid_stop_edit(
                grids,
                nodes,
                text_state,
                interaction_state,
                commands,
                entity,
                commit,
            );
        }
    }

    handle_data_grid_column_resize(window, grids, nodes, entity, data, context);

    let mut is_focused = false;
    grids.for_entity(entity, |(grid,)| is_focused = grid.focused);

    if is_focused && data.total_rows > 0 {
        handle_data_grid_keyboard(grids, scroll_areas, entity, data, context);
    }

    update_data_grid_visibility(grids, nodes, weights, scroll_areas, entity, data);
}

fn data_grid_update_sort_indicators(
    grids: &mut UiDataGrids<'_, '_>,
    text_state: &mut nightshade_text::contract::resources::TextState,
    entity: nightshade::ecs::Entity,
    data: &crate::components::UiDataGridData,
) {
    let mut sort = None;
    grids.for_entity(entity, |(grid,)| {
        sort = Some((grid.sort_column, grid.sort_ascending))
    });
    let Some((sort_column, sort_ascending)) = sort else {
        return;
    };

    for (index, text_slot) in data.header_text_slots.iter().enumerate() {
        let base_label = &data.columns[index].label;
        let display = match sort_column {
            Some(column) if column == index => {
                let indicator = if sort_ascending {
                    " \u{25B2}"
                } else {
                    " \u{25BC}"
                };
                format!("{base_label}{indicator}")
            }
            _ => base_label.clone(),
        };
        text_state.cache.set_text(*text_slot, &display);
    }
}

pub fn ui_data_grid_system(
    input: Res<nightshade_platform::contract::input::Input>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    mut window: ResMut<nightshade_platform::contract::window::Window>,
    mut interaction_state: ResMut<crate::resources::RetainedUiInteraction>,
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    mut commands: Commands,
    mut grids: UiDataGrids<'_, '_>,
    mut nodes: UiEditNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
    mut scroll_areas: UiScrollAreas<'_, '_>,
    mut weights: UiWeights<'_, '_>,
) {
    let state = input.mouse.state;
    let context = DataGridContext {
        ctrl_held: mirror.ctrl_held,
        shift_held: mirror.shift_held,
        frame_keys: &mirror.frame_keys,
        mouse_position: input.mouse.position,
        mouse_just_pressed: state
            .contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_PRESSED),
        mouse_down: state.contains(nightshade_platform::contract::input::MouseState::LEFT_CLICKED),
        viewport_size: window
            .cached_viewport_size
            .map(|(width, height)| Vec2::new(width as f32, height as f32))
            .unwrap_or(Vec2::new(800.0, 600.0)),
    };

    let mut pending = Vec::new();
    grids.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        handle_data_grid(
            &mut window,
            &mut interaction_state,
            &mut text_state,
            &mut commands,
            &mut grids,
            &mut nodes,
            &mut interactions,
            &mut scroll_areas,
            &mut weights,
            entity,
            &data,
            &context,
        );
    }
}

pub fn ui_data_grid_start_edit(
    grids: &mut UiDataGrids<'_, '_>,
    nodes: &mut UiEditNodes<'_, '_>,
    text_state: &mut nightshade_text::contract::resources::TextState,
    interaction_state: &mut crate::resources::RetainedUiInteraction,
    commands: &mut Commands,
    entity: nightshade::ecs::Entity,
    row: usize,
    column: usize,
) {
    let mut grid_data = None;
    grids.for_entity(entity, |(grid,)| grid_data = Some(grid.clone()));
    let Some(data) = grid_data else {
        return;
    };
    if column >= data.columns.len() || !data.columns[column].editable {
        return;
    }
    let Some(input_entity) = data.editing_input_entity else {
        return;
    };
    if row < data.visible_start {
        return;
    }
    let pool_row_index = row - data.visible_start;
    if pool_row_index >= data.pool_rows.len() {
        return;
    }
    let cell_entity = data.pool_rows[pool_row_index].cell_entities[column];
    let cell_text_slot = data.pool_rows[pool_row_index].cell_text_slots[column];
    let cell_text = text_state
        .cache
        .get_text(cell_text_slot)
        .unwrap_or_default()
        .to_string();

    let root_rect = edit_node_rect(nodes, entity).unwrap_or_default();
    let cell_rect = edit_node_rect(nodes, cell_entity).unwrap_or_default();
    let relative = cell_rect.min - root_rect.min;
    let size = Vec2::new(cell_rect.width(), cell_rect.height());

    nodes.for_entity(input_entity, |(node, input_data)| {
        if let Some(input_data) = input_data {
            input_data.text = cell_text;
            input_data.selection_start = Some(0);
        }
        node.visible = true;
        if let Some(crate::components::layout_types::UiLayoutType::Window(window)) =
            node.base_layout.as_mut()
        {
            window.position = Ab(relative).into();
            window.size = Ab(size).into();
        }
    });

    interaction_state.focused_entity = Some(input_entity);
    grids.for_entity(entity, |(grid,)| grid.editing_cell = Some((row, column)));
    let _ = commands;
}

pub fn ui_data_grid_stop_edit(
    grids: &mut UiDataGrids<'_, '_>,
    nodes: &mut UiEditNodes<'_, '_>,
    text_state: &mut nightshade_text::contract::resources::TextState,
    interaction_state: &mut crate::resources::RetainedUiInteraction,
    commands: &mut Commands,
    entity: nightshade::ecs::Entity,
    commit: bool,
) {
    let _ = text_state;
    let mut cell = None;
    grids.for_entity(entity, |(grid,)| {
        cell = grid.editing_cell.zip(grid.editing_input_entity)
    });
    let Some(((row, column), input_entity)) = cell else {
        return;
    };

    let mut text = String::new();
    nodes.for_entity(input_entity, |(node, input_data)| {
        if let Some(input_data) = input_data {
            text = input_data.text.clone();
        }
        node.visible = false;
    });

    if commit {
        commands.queue(move |_, resources| {
            resources
                .res_mut::<crate::resources::RetainedUiFrame>()
                .events
                .push(crate::resources::UiEvent::DataGridCellEdited {
                    entity,
                    row,
                    column,
                    text,
                });
        });
    }

    if interaction_state.focused_entity == Some(input_entity) {
        interaction_state.focused_entity = None;
    }
    grids.for_entity(entity, |(grid,)| grid.editing_cell = None);
}

fn handle_command_palette(
    interaction_state: &mut crate::resources::RetainedUiInteraction,
    frame: &mut crate::resources::RetainedUiFrame,
    commands: &mut Commands,
    palettes: &mut UiCommandPalettes<'_, '_>,
    nodes: &mut UiEditNodes<'_, '_>,
    interactions: &mut UiInteractions<'_, '_>,
    colors: &mut UiColors<'_, '_>,
    accent_color: Vec4,
    frame_keys: &[(KeyCode, bool)],
    entity: nightshade::ecs::Entity,
    data: &crate::components::UiCommandPaletteData,
) {
    if !data.open {
        palettes.for_entity(entity, |(palette,)| palette.executed_command = None);
        return;
    }

    let mut text_changed = false;
    let mut new_text = String::new();
    nodes.for_entity(data.text_input_entity, |(_, input_data)| {
        if let Some(input_data) = input_data {
            text_changed = input_data.changed;
            new_text = input_data.text.clone();
        }
    });

    if text_changed {
        let filter_lower = new_text.to_lowercase();
        let new_indices: Vec<usize> = data
            .commands
            .iter()
            .enumerate()
            .filter(|(_, command)| {
                filter_lower.is_empty()
                    || command.label.to_lowercase().contains(&filter_lower)
                    || command.category.to_lowercase().contains(&filter_lower)
            })
            .map(|(index, _)| index)
            .collect();

        palettes.for_entity(entity, |(palette,)| {
            palette.filter_text = new_text.clone();
            palette.filtered_indices = new_indices.clone();
            palette.selected_index = 0;
        });

        commands.queue(move |storage, resources| {
            nightshade::ecs::with_commands_over(
                storage,
                resources,
                |storage, resources, commands| {
                    let mut context =
                        crate::commands::builder::ui_build_context(resources, commands);
                    ui_command_palette_rebuild_results(&mut context, storage, entity);
                },
            );
        });
    }

    let filtered_count = data.filtered_indices.len();
    let mut execute_index = None;
    let mut should_close = false;

    for &(key, pressed) in frame_keys {
        if !pressed {
            continue;
        }
        match key {
            KeyCode::ArrowUp => {
                palettes.for_entity(entity, |(palette,)| {
                    palette.selected_index = palette.selected_index.saturating_sub(1)
                });
            }
            KeyCode::ArrowDown if filtered_count > 0 => {
                palettes.for_entity(entity, |(palette,)| {
                    palette.selected_index = (palette.selected_index + 1).min(filtered_count - 1)
                });
            }
            KeyCode::Enter if filtered_count > 0 => {
                let mut selected = 0;
                palettes.for_entity(entity, |(palette,)| selected = palette.selected_index);
                if selected < filtered_count {
                    execute_index = Some(data.filtered_indices[selected]);
                }
            }
            KeyCode::Escape => {
                should_close = true;
            }
            _ => {}
        }
    }

    for (pool_index, &row_entity) in data.result_entities.iter().enumerate() {
        if pool_index < filtered_count && row_interaction(interactions, row_entity).clicked {
            execute_index = Some(data.filtered_indices[pool_index]);
        }
    }

    let mut selected_index = 0;
    palettes.for_entity(entity, |(palette,)| selected_index = palette.selected_index);

    for (pool_index, &row_entity) in data.result_entities.iter().enumerate() {
        if pool_index >= filtered_count {
            continue;
        }
        let background = if pool_index == selected_index {
            Vec4::new(accent_color.x, accent_color.y, accent_color.z, 0.3)
        } else {
            Vec4::new(0.0, 0.0, 0.0, 0.0)
        };
        colors.for_entity(row_entity, |(color,)| {
            color.colors[crate::data::state::UiBase::INDEX] = Some(background)
        });
    }

    if row_interaction(interactions, data.backdrop_entity).clicked || execute_index.is_some() {
        should_close = true;
    }

    if !should_close {
        return;
    }

    nodes.for_entity(data.backdrop_entity, |(node, _)| node.visible = false);
    nodes.for_entity(entity, |(node, _)| node.visible = false);
    palettes.for_entity(entity, |(palette,)| {
        palette.open = false;
        palette.executed_command = execute_index;
    });
    if interaction_state.focused_entity == Some(data.text_input_entity) {
        interaction_state.focused_entity = None;
    }

    if let Some(command_index) = execute_index {
        frame
            .events
            .push(crate::resources::UiEvent::CommandPaletteExecuted {
                entity,
                command_index,
            });
    }
}

pub fn ui_command_palette_system(
    theme: Res<crate::resources::ThemeState>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    mut interaction_state: ResMut<crate::resources::RetainedUiInteraction>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut commands: Commands,
    mut palettes: UiCommandPalettes<'_, '_>,
    mut nodes: UiEditNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
    mut colors: UiColors<'_, '_>,
) {
    let accent_color = theme.active_theme().accent_color;

    let mut pending = Vec::new();
    palettes.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        handle_command_palette(
            &mut interaction_state,
            &mut frame,
            &mut commands,
            &mut palettes,
            &mut nodes,
            &mut interactions,
            &mut colors,
            accent_color,
            &mirror.frame_keys,
            entity,
            &data,
        );
    }
}

fn handle_property_grid(
    window: &mut nightshade_platform::contract::window::Window,
    grids: &mut UiPropertyGrids<'_, '_>,
    nodes: &mut UiNodes<'_, '_>,
    entity: nightshade::ecs::Entity,
    data: &crate::components::UiPropertyGridData,
    context: &DataGridContext<'_>,
) {
    let mut resize = None;
    grids.for_entity(entity, |(grid,)| {
        resize = Some((
            grid.resize_active,
            grid.resize_start_x,
            grid.resize_start_width,
        ))
    });
    let Some((currently_resizing, start_x, start_width)) = resize else {
        return;
    };

    if currently_resizing {
        window.requested_cursor = Some(nightshade_platform::CursorIcon::ColResize);

        if !context.mouse_down {
            grids.for_entity(entity, |(grid,)| grid.resize_active = false);
            return;
        }

        let grid_rect = node_rect(nodes, entity);
        let grid_width = grid_rect.map(|rect| rect.width()).unwrap_or(300.0);
        let grid_height = grid_rect.map(|rect| rect.height()).unwrap_or(0.0);

        let delta = context.mouse_position.x - start_x;
        let new_width = (start_width + delta).clamp(30.0, grid_width - 50.0);

        grids.for_entity(entity, |(grid,)| grid.label_width = new_width);

        for label_entity in &data.label_entities {
            nodes.for_entity(*label_entity, |(node,)| {
                let Some(ref mut child_size) = node.flow_child_size else {
                    return;
                };
                let mut evaluated = child_size.evaluate(&crate::data::units::UiEvalContext {
                    parent_width: grid_width,
                    parent_height: grid_height,
                    viewport_width: context.viewport_size.x,
                    viewport_height: context.viewport_size.y,
                    font_size: 16.0,
                    absolute_scale: 1.0,
                });
                evaluated.x = new_width;
                *child_size = crate::data::units::Ab(evaluated).into();
            });
        }
        return;
    }

    let edge_threshold = 5.0;
    let mut near_edge = false;

    for label_entity in &data.label_entities {
        let Some(rect) = node_rect(nodes, *label_entity) else {
            continue;
        };
        let near_right_edge = (context.mouse_position.x - rect.max.x).abs() < edge_threshold
            && context.mouse_position.y >= rect.min.y
            && context.mouse_position.y <= rect.max.y;
        if !near_right_edge {
            continue;
        }
        near_edge = true;
        if context.mouse_just_pressed {
            grids.for_entity(entity, |(grid,)| {
                grid.resize_active = true;
                grid.resize_start_x = context.mouse_position.x;
                grid.resize_start_width = grid.label_width;
            });
            return;
        }
        break;
    }

    if near_edge {
        window.requested_cursor = Some(nightshade_platform::CursorIcon::ColResize);
    }
}

pub fn ui_property_grid_system(
    input: Res<nightshade_platform::contract::input::Input>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    mut window: ResMut<nightshade_platform::contract::window::Window>,
    mut grids: UiPropertyGrids<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
) {
    let state = input.mouse.state;
    let context = DataGridContext {
        ctrl_held: mirror.ctrl_held,
        shift_held: mirror.shift_held,
        frame_keys: &mirror.frame_keys,
        mouse_position: input.mouse.position,
        mouse_just_pressed: state
            .contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_PRESSED),
        mouse_down: state.contains(nightshade_platform::contract::input::MouseState::LEFT_CLICKED),
        viewport_size: window
            .cached_viewport_size
            .map(|(width, height)| Vec2::new(width as f32, height as f32))
            .unwrap_or(Vec2::new(800.0, 600.0)),
    };

    let mut pending = Vec::new();
    grids.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        handle_property_grid(&mut window, &mut grids, &mut nodes, entity, &data, &context);
    }
}

fn handle_data_grid_column_resize(
    window: &mut nightshade_platform::contract::window::Window,
    grids: &mut UiDataGrids<'_, '_>,
    nodes: &mut UiEditNodes<'_, '_>,
    entity: nightshade::ecs::Entity,
    data: &crate::components::UiDataGridData,
    context: &DataGridContext<'_>,
) {
    let mut resize = None;
    grids.for_entity(entity, |(grid,)| {
        resize = Some((
            grid.resize_column,
            grid.resize_start_x,
            grid.resize_start_width,
        ))
    });
    let Some((currently_resizing, start_x, start_width)) = resize else {
        return;
    };

    if let Some(resize_column) = currently_resizing {
        window.requested_cursor = Some(nightshade_platform::CursorIcon::ColResize);

        if !context.mouse_down {
            grids.for_entity(entity, |(grid,)| grid.resize_column = None);
            return;
        }

        let delta = context.mouse_position.x - start_x;
        let new_width = (start_width + delta).max(30.0);

        grids.for_entity(entity, |(grid,)| {
            if let Some(column) = grid.columns.get_mut(resize_column) {
                column.width = new_width;
            }
        });

        let mut resized = Vec::new();
        if let Some(header_entity) = data.header_entities.get(resize_column) {
            resized.push(*header_entity);
        }
        for pool_row in &data.pool_rows {
            if let Some(&cell_entity) = pool_row.cell_entities.get(resize_column) {
                resized.push(cell_entity);
            }
        }
        for target in resized {
            nodes.for_entity(target, |(node, _)| {
                let Some(ref mut child_size) = node.flow_child_size else {
                    return;
                };
                let mut evaluated = child_size.evaluate(&crate::data::units::UiEvalContext {
                    parent_width: context.viewport_size.x,
                    parent_height: context.viewport_size.y,
                    viewport_width: context.viewport_size.x,
                    viewport_height: context.viewport_size.y,
                    font_size: 16.0,
                    absolute_scale: 1.0,
                });
                evaluated.x = new_width;
                *child_size = crate::data::units::Ab(evaluated).into();
            });
        }
        return;
    }

    let edge_threshold = 5.0;
    let mut near_any_edge = false;

    for (column_index, header_entity) in data.header_entities.iter().enumerate() {
        let Some(rect) = edit_node_rect(nodes, *header_entity) else {
            continue;
        };
        let near_right_edge = (context.mouse_position.x - rect.max.x).abs() < edge_threshold
            && context.mouse_position.y >= rect.min.y
            && context.mouse_position.y <= rect.max.y;
        if !near_right_edge {
            continue;
        }
        near_any_edge = true;
        if context.mouse_just_pressed {
            grids.for_entity(entity, |(grid,)| {
                grid.resize_column = Some(column_index);
                grid.resize_start_x = context.mouse_position.x;
                grid.resize_start_width = data.columns[column_index].width;
            });
            return;
        }
        break;
    }

    if near_any_edge {
        window.requested_cursor = Some(nightshade_platform::CursorIcon::ColResize);
    }
}

fn handle_data_grid_keyboard(
    grids: &mut UiDataGrids<'_, '_>,
    scroll_areas: &mut UiScrollAreas<'_, '_>,
    entity: nightshade::ecs::Entity,
    data: &crate::components::UiDataGridData,
    context: &DataGridContext<'_>,
) {
    let total_rows = data
        .filtered_indices
        .as_ref()
        .map_or(data.total_rows, |indices| indices.len());
    if total_rows == 0 {
        return;
    }

    let mut current_anchor = None;
    let mut found = false;
    grids.for_entity(entity, |(grid,)| {
        current_anchor = grid.selection_anchor;
        found = true;
    });
    if !found {
        return;
    }

    let anchor = current_anchor.unwrap_or(0);
    let mut new_anchor = None;

    for &(key, pressed) in context.frame_keys {
        if !pressed {
            continue;
        }
        match key {
            KeyCode::ArrowUp => new_anchor = Some(anchor.saturating_sub(1)),
            KeyCode::ArrowDown => new_anchor = Some((anchor + 1).min(total_rows - 1)),
            KeyCode::PageUp => new_anchor = Some(anchor.saturating_sub(data.pool_size)),
            KeyCode::PageDown => new_anchor = Some((anchor + data.pool_size).min(total_rows - 1)),
            KeyCode::Home => new_anchor = Some(0),
            KeyCode::End => new_anchor = Some(total_rows - 1),
            KeyCode::KeyA if context.ctrl_held => {
                grids.for_entity(entity, |(grid,)| {
                    grid.selected_rows.clear();
                    for row in 0..total_rows {
                        grid.selected_rows.insert(row);
                    }
                    grid.selection_changed = true;
                });
                return;
            }
            _ => {}
        }
    }

    let Some(target) = new_anchor else {
        return;
    };

    grids.for_entity(entity, |(grid,)| {
        if context.shift_held {
            grid.selected_rows.clear();
            for row in anchor.min(target)..=anchor.max(target) {
                grid.selected_rows.insert(row);
            }
        } else {
            grid.selected_rows.clear();
            grid.selected_rows.insert(target);
            grid.selection_anchor = Some(target);
        }
        grid.selection_changed = true;
    });

    let row_height = data.row_height;
    let target_offset = target as f32 * row_height;
    scroll_areas.for_entity(data.scroll_entity, |(scroll_data,)| {
        if target_offset < scroll_data.scroll_offset {
            scroll_data.scroll_offset = target_offset;
        } else if target_offset + row_height
            > scroll_data.scroll_offset + scroll_data.visible_height
        {
            scroll_data.scroll_offset = target_offset + row_height - scroll_data.visible_height;
        }
    });
}

fn update_data_grid_visibility(
    grids: &mut UiDataGrids<'_, '_>,
    nodes: &mut UiEditNodes<'_, '_>,
    weights: &mut UiWeights<'_, '_>,
    scroll_areas: &mut UiScrollAreas<'_, '_>,
    entity: nightshade::ecs::Entity,
    data: &crate::components::UiDataGridData,
) {
    let mut scroll_offset = 0.0;
    scroll_areas.for_entity(data.scroll_entity, |(scroll_data,)| {
        scroll_offset = scroll_data.scroll_offset
    });

    let effective_row_count = data
        .filtered_indices
        .as_ref()
        .map_or(data.total_rows, |indices| indices.len());

    let window = super::pool_window::compute(
        scroll_offset,
        data.row_height,
        effective_row_count,
        data.pool_size,
    );
    let visible_start = window.visible_start;

    nodes.for_entity(data.top_spacer, |(node, _)| {
        node.flow_child_size =
            Some(crate::data::units::Ab(nalgebra_glm::Vec2::new(0.0, window.top_height)).into())
    });
    nodes.for_entity(data.bottom_spacer, |(node, _)| {
        node.flow_child_size =
            Some(crate::data::units::Ab(nalgebra_glm::Vec2::new(0.0, window.bottom_height)).into())
    });

    let mut selected_rows = std::collections::HashSet::new();
    grids.for_entity(entity, |(grid,)| selected_rows = grid.selected_rows.clone());

    for (pool_index, pool_row) in data.pool_rows.iter().enumerate() {
        let visible_row = visible_start + pool_index;
        let visible = visible_row < effective_row_count;

        nodes.for_entity(pool_row.row_entity, |(node, _)| node.visible = visible);

        if visible {
            let is_selected = selected_rows.contains(&visible_row);
            weights.for_entity(pool_row.row_entity, |(weight,)| {
                weight.weights[crate::data::state::UiSelected::INDEX] =
                    if is_selected { 1.0 } else { 0.0 }
            });
        }
    }

    grids.for_entity(entity, |(grid,)| grid.visible_start = visible_start);
}
