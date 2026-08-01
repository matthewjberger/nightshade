use nalgebra_glm::Vec2;
use nightshade::ecs::system_param::{Commands, Query, Res, ResMut};
use nightshade_platform::KeyCode;

use super::handle_frame::{UiDropdowns, UiMenus};
use super::handle_scroll::{UiInteractions, UiNodes};
use super::handle_slider::row_interaction;
use crate::data::state::UiStateTrait as _;

use crate::prelude::*;

pub type UiTabBars<'world, 'item> = Query<'world, (&'item mut crate::components::UiTabBarData,)>;

pub type UiMultiSelects<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiMultiSelectData,)>;

pub type UiDatePickers<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiDatePickerData,)>;

pub type UiColors<'world, 'item> = Query<'world, (&'item mut crate::components::UiNodeColor,)>;

pub type UiWeights<'world, 'item> = Query<'world, (&'item mut crate::components::UiStateWeights,)>;

pub type UiTextSlots<'world, 'item> = Query<'world, (&'item crate::components::UiNodeContent,)>;

pub type UiFilterInputs<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiTextInputData,)>;

#[derive(Clone, Copy)]
struct PointerFrame {
    position: Vec2,
    left_just_pressed: bool,
}

fn mouse_inside_visible(
    nodes: &mut UiNodes<'_, '_>,
    entity: nightshade::ecs::Entity,
    mouse_position: Vec2,
) -> bool {
    let mut inside = false;
    nodes.for_entity(entity, |(node,)| {
        inside = node.visible && node.computed_rect.contains(mouse_position)
    });
    inside
}

fn dismiss_popup_on_outside_press(
    nodes: &mut UiNodes<'_, '_>,
    pointer: PointerFrame,
    owner_hovered: bool,
    popup_entity: nightshade::ecs::Entity,
) -> bool {
    !owner_hovered
        && pointer.left_just_pressed
        && !mouse_inside_visible(nodes, popup_entity, pointer.position)
}

fn set_popup_visible(
    nodes: &mut UiNodes<'_, '_>,
    entity: nightshade::ecs::Entity,
    visible: bool,
) -> bool {
    let mut changed = false;
    nodes.for_entity(entity, |(node,)| {
        changed = node.visible != visible;
        node.visible = visible;
    });
    changed
}

pub fn ui_tab_bar_system(
    theme: Res<crate::resources::ThemeState>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    interaction_state: Res<crate::resources::RetainedUiInteraction>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut tab_bars: UiTabBars<'_, '_>,
    mut colors: UiColors<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
) {
    let active_theme = theme.active_theme();
    let active_background = active_theme.accent_color;
    let inactive_background = active_theme.background_color;
    let focused_entity = interaction_state.focused_entity;

    let mut pending = Vec::new();
    tab_bars.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        let mut new_selected = data.selected_tab;
        let mut changed = false;

        for (index, tab_entity) in data.tab_entities.iter().enumerate() {
            if row_interaction(&mut interactions, *tab_entity).clicked && data.selected_tab != index
            {
                new_selected = index;
                changed = true;
                break;
            }
        }

        let any_tab_focused = data
            .tab_entities
            .iter()
            .any(|tab| focused_entity == Some(*tab));
        if any_tab_focused {
            let tab_count = data.tab_entities.len();
            for &(key, pressed) in &mirror.frame_keys {
                if !pressed {
                    continue;
                }
                match key {
                    KeyCode::ArrowLeft if new_selected > 0 => {
                        new_selected -= 1;
                        changed = true;
                    }
                    KeyCode::ArrowRight if new_selected + 1 < tab_count => {
                        new_selected += 1;
                        changed = true;
                    }
                    _ => {}
                }
            }
        }

        if changed {
            if let Some(old_entity) = data.tab_entities.get(data.selected_tab).copied() {
                colors.for_entity(old_entity, |(color,)| {
                    color.colors[crate::data::state::UiBase::INDEX] = Some(inactive_background)
                });
            }
            if let Some(new_entity) = data.tab_entities.get(new_selected).copied() {
                colors.for_entity(new_entity, |(color,)| {
                    color.colors[crate::data::state::UiBase::INDEX] = Some(active_background)
                });
            }
        }

        tab_bars.for_entity(entity, |(widget_data,)| {
            widget_data.selected_tab = new_selected;
            widget_data.changed = changed;
        });
        if changed {
            frame.events.push(crate::resources::UiEvent::TabChanged {
                entity,
                tab_index: new_selected,
            });
        }
    }
}

pub fn ui_dropdown_system(
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    input: Res<nightshade_platform::contract::input::Input>,
    mut theme: ResMut<crate::resources::ThemeState>,
    mut dirty: ResMut<crate::resources::RetainedUiDirty>,
    mut interaction_state: ResMut<crate::resources::RetainedUiInteraction>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    mut dropdowns: UiDropdowns<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
    mut weights: UiWeights<'_, '_>,
    mut filter_inputs: UiFilterInputs<'_, '_>,
) {
    let focused_entity = interaction_state.focused_entity;
    let mouse_just_pressed = input
        .mouse
        .state
        .contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_PRESSED);

    let mut pending = Vec::new();
    dropdowns.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        let interaction = row_interaction(&mut interactions, entity);
        let mut selected_index = data.selected_index;
        let mut changed = false;
        let mut open = data.open;
        let mut hovered_index = data.hovered_index;
        let was_open = data.open;
        let option_count = data.options.len();
        let searchable = data.searchable;
        let filter_input_entity = data.filter_input_entity;
        let mut filtered_indices = data.filtered_indices.clone();
        let mut filter_text = data.filter_text.clone();

        if interaction.clicked {
            open = !open;
        }

        if !open && focused_entity == Some(entity) {
            for &(key, pressed) in &mirror.frame_keys {
                if pressed
                    && (key == KeyCode::ArrowDown || key == KeyCode::Space || key == KeyCode::Enter)
                {
                    open = true;
                    hovered_index = Some(selected_index);
                    break;
                }
            }
        }

        if open
            && !was_open
            && searchable
            && let Some(filter_entity) = filter_input_entity
        {
            let mut text_slot = None;
            filter_inputs.for_entity(filter_entity, |(input_data,)| {
                input_data.text.clear();
                input_data.cursor_position = 0;
                input_data.selection_start = None;
                input_data.changed = false;
                text_slot = Some(input_data.text_slot);
            });
            if let Some(text_slot) = text_slot {
                text_state.cache.set_text(text_slot, "");
            }
            interaction_state.focused_entity = Some(filter_entity);
            filter_text.clear();
            filtered_indices = (0..option_count).collect();
        }

        if open
            && searchable
            && let Some(filter_entity) = filter_input_entity
        {
            let mut current_filter = String::new();
            filter_inputs.for_entity(filter_entity, |(input_data,)| {
                current_filter = input_data.text.clone()
            });

            if current_filter != filter_text {
                filter_text = current_filter;
                let lower_filter = filter_text.to_lowercase();
                filtered_indices = (0..option_count)
                    .filter(|option_index| {
                        lower_filter.is_empty()
                            || data.options[*option_index]
                                .to_lowercase()
                                .contains(&lower_filter)
                    })
                    .collect();
                hovered_index = None;

                for (index, popup_entity) in data.popup_entities.iter().enumerate() {
                    let visible = filtered_indices.contains(&index);
                    nodes.for_entity(*popup_entity, |(node,)| node.visible = visible);
                }
            }
        }

        if open {
            let visible_count = if searchable {
                filtered_indices.len()
            } else {
                option_count
            };

            for &(key, pressed) in &mirror.frame_keys {
                if !pressed {
                    continue;
                }
                match key {
                    KeyCode::ArrowDown if visible_count > 0 => {
                        let current_position =
                            visible_position(hovered_index, &filtered_indices, searchable);
                        let next_position = if current_position + 1 >= visible_count {
                            0
                        } else {
                            current_position + 1
                        };
                        hovered_index = Some(if searchable {
                            filtered_indices[next_position]
                        } else {
                            next_position
                        });
                    }
                    KeyCode::ArrowUp if visible_count > 0 => {
                        let current_position =
                            visible_position(hovered_index, &filtered_indices, searchable);
                        let previous_position = if current_position == 0 {
                            visible_count.saturating_sub(1)
                        } else {
                            current_position - 1
                        };
                        hovered_index = Some(if searchable {
                            filtered_indices[previous_position]
                        } else {
                            previous_position
                        });
                    }
                    KeyCode::Enter => {
                        if let Some(option_index) = hovered_index
                            && let Some(selected_text) = data.options.get(option_index).cloned()
                        {
                            selected_index = option_index;
                            changed = true;
                            open = false;
                            text_state
                                .cache
                                .set_text(data.header_text_slot, selected_text);
                        }
                    }
                    KeyCode::Escape => {
                        open = false;
                    }
                    _ => {}
                }
            }

            for (index, popup_entity) in data.popup_entities.iter().enumerate() {
                if searchable && !filtered_indices.contains(&index) {
                    continue;
                }
                let popup = row_interaction(&mut interactions, *popup_entity);
                if popup.hovered {
                    hovered_index = Some(index);
                }
                if popup.clicked
                    && let Some(selected_text) = data.options.get(index).cloned()
                {
                    selected_index = index;
                    changed = true;
                    open = false;
                    text_state
                        .cache
                        .set_text(data.header_text_slot, selected_text);
                    break;
                }
            }

            for (index, popup_entity) in data.popup_entities.iter().enumerate() {
                let is_highlighted = hovered_index == Some(index);
                weights.for_entity(*popup_entity, |(weight,)| {
                    weight.weights[crate::data::state::UiHover::INDEX] =
                        if is_highlighted { 1.0 } else { 0.0 }
                });
            }

            let filter_hovered = filter_input_entity
                .is_some_and(|filter| row_interaction(&mut interactions, filter).hovered);
            let mouse_over_popup = data
                .popup_entities
                .iter()
                .any(|popup| row_interaction(&mut interactions, *popup).hovered);
            if mouse_just_pressed && !interaction.hovered && !mouse_over_popup && !filter_hovered {
                open = false;
            }
        }

        if searchable && was_open && !open {
            if let Some(filter_entity) = filter_input_entity
                && interaction_state.focused_entity == Some(filter_entity)
            {
                interaction_state.focused_entity = None;
            }
            for popup_entity in &data.popup_entities {
                nodes.for_entity(*popup_entity, |(node,)| node.visible = true);
            }
        }

        if data.is_theme_dropdown {
            if open && hovered_index != data.hovered_index {
                theme.set_preview(hovered_index);
            }
            if was_open && !open {
                theme.clear_preview();
            }
            if changed {
                theme.select_theme(selected_index);
            }
        }

        if set_popup_visible(&mut nodes, data.popup_container_entity, open) {
            ui_mark_layout_dirty(&mut dirty);
        }

        dropdowns.for_entity(entity, |(widget_data,)| {
            widget_data.selected_index = selected_index;
            widget_data.changed = changed;
            widget_data.open = open;
            widget_data.hovered_index = hovered_index;
            widget_data.filter_text = filter_text.clone();
            widget_data.filtered_indices = filtered_indices.clone();
        });
        if changed {
            frame
                .events
                .push(crate::resources::UiEvent::DropdownChanged {
                    entity,
                    selected_index,
                });
        }
    }
}

fn visible_position(
    hovered_index: Option<usize>,
    filtered_indices: &[usize],
    searchable: bool,
) -> usize {
    hovered_index
        .and_then(|hovered_option| {
            if searchable {
                filtered_indices
                    .iter()
                    .position(|&option_index| option_index == hovered_option)
            } else {
                Some(hovered_option)
            }
        })
        .unwrap_or(0)
}

pub fn ui_multi_select_system(
    input: Res<nightshade_platform::contract::input::Input>,
    mut dirty: ResMut<crate::resources::RetainedUiDirty>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    mut multi_selects: UiMultiSelects<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
    mut contents: UiTextSlots<'_, '_>,
) {
    let pointer = PointerFrame {
        position: input.mouse.position,
        left_just_pressed: input
            .mouse
            .state
            .contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_PRESSED),
    };

    let mut pending = Vec::new();
    multi_selects.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        let interaction = row_interaction(&mut interactions, entity);
        let mut selected_indices = data.selected_indices.clone();
        let mut changed = false;
        let mut open = data.open;
        let mut hovered_index = data.hovered_index;

        if interaction.clicked {
            open = !open;
        }

        if open {
            for (index, &popup_entity) in data.popup_entities.iter().enumerate() {
                let popup = row_interaction(&mut interactions, popup_entity);
                if popup.clicked {
                    if selected_indices.contains(&index) {
                        selected_indices.remove(&index);
                    } else {
                        selected_indices.insert(index);
                    }
                    changed = true;
                }
                if popup.hovered {
                    hovered_index = Some(index);
                }
            }

            if dismiss_popup_on_outside_press(
                &mut nodes,
                pointer,
                interaction.hovered,
                data.popup_container_entity,
            ) {
                open = false;
            }
        }

        let header_text = if selected_indices.is_empty() {
            "None selected".to_string()
        } else if selected_indices.len() == 1 {
            let option_index = *selected_indices.iter().next().unwrap();
            data.options.get(option_index).cloned().unwrap_or_default()
        } else {
            format!("{} selected", selected_indices.len())
        };
        text_state
            .cache
            .set_text(data.header_text_slot, &header_text);

        for (index, &check_entity) in data.check_entities.iter().enumerate() {
            let check_text = if selected_indices.contains(&index) {
                "\u{2713}"
            } else {
                " "
            };
            let mut slot = None;
            contents.for_entity(check_entity, |(content,)| {
                if let crate::components::UiNodeContent::Text { text_slot, .. } = content {
                    slot = Some(*text_slot);
                }
            });
            if let Some(slot) = slot {
                text_state.cache.set_text(slot, check_text);
            }
        }

        if set_popup_visible(&mut nodes, data.popup_container_entity, open) {
            ui_mark_layout_dirty(&mut dirty);
        }

        multi_selects.for_entity(entity, |(widget_data,)| {
            widget_data.selected_indices = selected_indices.clone();
            widget_data.changed = changed;
            widget_data.open = open;
            widget_data.hovered_index = hovered_index;
        });
        if changed {
            frame
                .events
                .push(crate::resources::UiEvent::MultiSelectChanged {
                    entity,
                    selected_indices: selected_indices.into_iter().collect(),
                });
        }
    }
}

pub fn ui_date_picker_system(
    theme: Res<crate::resources::ThemeState>,
    input: Res<nightshade_platform::contract::input::Input>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    mut commands: Commands,
    mut pickers: UiDatePickers<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
) {
    let accent_color = theme.active_theme().accent_color;
    let pointer = PointerFrame {
        position: input.mouse.position,
        left_just_pressed: input
            .mouse
            .state
            .contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_PRESSED),
    };

    let mut pending = Vec::new();
    pickers.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        let interaction = row_interaction(&mut interactions, entity);
        let mut year = data.year;
        let mut month = data.month;
        let mut day = data.day;
        let mut changed = false;
        let mut open = data.open;
        let mut repopulate = false;

        if interaction.clicked {
            open = !open;
        }

        if open {
            if row_interaction(&mut interactions, data.prev_month_entity).clicked {
                if month == 1 {
                    month = 12;
                    year -= 1;
                } else {
                    month -= 1;
                }
                repopulate = true;
            }

            if row_interaction(&mut interactions, data.next_month_entity).clicked {
                if month == 12 {
                    month = 1;
                    year += 1;
                } else {
                    month += 1;
                }
                repopulate = true;
            }

            let first_day_of_week = crate::commands::widgets::day_of_week(year, month, 1);
            let days = crate::commands::widgets::days_in_month(year, month);

            for (cell_index, &day_entity) in data.day_entities.iter().enumerate() {
                if !row_interaction(&mut interactions, day_entity).clicked {
                    continue;
                }
                let cell = cell_index as u32;
                if cell >= first_day_of_week && cell < first_day_of_week + days {
                    day = cell - first_day_of_week + 1;
                    changed = true;
                    open = false;
                }
            }

            if dismiss_popup_on_outside_press(
                &mut nodes,
                pointer,
                interaction.hovered,
                data.popup_entity,
            ) {
                open = false;
            }
        }

        if repopulate || changed {
            crate::commands::widgets::populate_calendar_grid(
                &mut text_state,
                &mut commands,
                year,
                month,
                day,
                &data.day_text_slots,
                &data.day_entities,
                accent_color,
            );
        }
        if repopulate {
            text_state.cache.set_text(
                data.month_label_slot,
                crate::commands::widgets::format_month_year(year, month),
            );
        }
        if changed {
            text_state.cache.set_text(
                data.header_text_slot,
                format!("{year:04}-{month:02}-{day:02}"),
            );
        }

        nodes.for_entity(data.popup_entity, |(node,)| node.visible = open);

        pickers.for_entity(entity, |(widget_data,)| {
            widget_data.year = year;
            widget_data.month = month;
            widget_data.day = day;
            widget_data.changed = changed;
            widget_data.open = open;
        });
        if changed {
            frame
                .events
                .push(crate::resources::UiEvent::DatePickerChanged {
                    entity,
                    year,
                    month,
                    day,
                });
        }
    }
}

pub fn ui_menu_system(
    input: Res<nightshade_platform::contract::input::Input>,
    mut dirty: ResMut<crate::resources::RetainedUiDirty>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut menus: UiMenus<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
) {
    let mouse_just_pressed = input
        .mouse
        .state
        .contains(nightshade_platform::contract::input::MouseState::LEFT_JUST_PRESSED);

    let mut pending = Vec::new();
    menus.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        let interaction = row_interaction(&mut interactions, entity);
        let mut clicked_item = None;
        let mut open = data.open;

        if interaction.clicked {
            open = !open;
        }

        if open {
            let mut any_popup_hovered = false;
            for (index, popup_entity) in data.popup_entities.iter().enumerate() {
                let popup = row_interaction(&mut interactions, *popup_entity);
                if popup.hovered {
                    any_popup_hovered = true;
                }
                if popup.clicked {
                    clicked_item = Some(index);
                    open = false;
                    break;
                }
            }

            if mouse_just_pressed && !interaction.hovered && !any_popup_hovered {
                open = false;
            }
        }

        if set_popup_visible(&mut nodes, data.popup_container_entity, open) {
            ui_mark_layout_dirty(&mut dirty);
        }

        menus.for_entity(entity, |(widget_data,)| {
            widget_data.clicked_item = clicked_item;
            widget_data.open = open;
        });
        if let Some(item_index) = clicked_item {
            frame
                .events
                .push(crate::resources::UiEvent::MenuItemClicked { entity, item_index });
        }
    }
}
