use nalgebra_glm;
use nalgebra_glm::Vec2;
use nightshade_platform::KeyCode;

use nightshade::ecs::system_param::{Query, Res, ResMut};

use super::handle_slider::RowInteraction;
use super::text_cursor::{byte_index_at_x, validate_widget_text};

pub type UiTextInputs<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiTextInputData,)>;

pub type UiSelectableLabels<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiSelectableLabelData,)>;
use super::text_edit::EditBuffer;
use crate::components::{CharStyle, TextSnapshot};
use crate::data::units::measure_text_width;

use crate::prelude::*;
fn handle_text_input(
    runtime: &mut crate::resources::RetainedUiRuntime,
    frame: &mut crate::resources::RetainedUiFrame,
    text_state: &mut nightshade_text::contract::resources::TextState,
    inputs: &mut UiTextInputs<'_, '_>,
    nodes: &mut super::handle_scroll::UiNodes<'_, '_>,
    interactions: &mut super::handle_scroll::UiInteractions<'_, '_>,
    font_size: f32,
    clear_focused: &mut bool,
    entity: nightshade::ecs::Entity,
    interaction: &RowInteraction,
    data: &crate::components::UiTextInputData,
    context: &super::TextEditContext<'_>,
) {
    let focused_entity = context.focused_entity;
    let frame_chars = context.frame_chars;
    let frame_keys = context.frame_keys;
    let ctrl_held = context.ctrl_held;
    let shift_held = context.shift_held;
    let mouse_position = context.mouse_position;
    let current_time = context.current_time;
    let dpi_scale = context.dpi_scale;
    let is_focused = focused_entity == Some(entity);
    let mut changed = false;
    let mut cursor_blink_timer = data.cursor_blink_timer;
    let mut scroll_offset = data.scroll_offset;
    let mut clear_focus = false;
    let mut submitted = false;
    let mut undo_stack = data.undo_stack.clone();
    let input_mask = &data.input_mask;
    let max_length = data.max_length;

    let mut buffer = EditBuffer::new(
        data.text.clone(),
        data.cursor_position,
        data.selection_start,
        false,
    );

    if is_focused {
        let mut needs_snapshot = false;
        for character in frame_chars {
            if *character >= ' ' && input_mask.accepts(*character) {
                if let Some(max) = max_length {
                    let removed = buffer
                        .selection_range()
                        .map(|(min, max_sel)| max_sel - min)
                        .unwrap_or(0);
                    if buffer.length() - removed + 1 > max {
                        continue;
                    }
                }
                if !needs_snapshot {
                    std::sync::Arc::make_mut(&mut undo_stack).push_initial(snapshot(&buffer));
                    needs_snapshot = true;
                }
                buffer.insert_char(*character, CharStyle::default());
                changed = true;
                cursor_blink_timer = current_time;
            }
        }

        if needs_snapshot {
            std::sync::Arc::make_mut(&mut undo_stack).push(snapshot(&buffer), current_time);
        }

        for (key, is_pressed) in frame_keys {
            if !is_pressed {
                continue;
            }
            match key {
                KeyCode::Backspace => {
                    std::sync::Arc::make_mut(&mut undo_stack).push_initial(snapshot(&buffer));
                    if buffer.backspace(ctrl_held) {
                        std::sync::Arc::make_mut(&mut undo_stack)
                            .push(snapshot(&buffer), current_time);
                        changed = true;
                    }
                    cursor_blink_timer = current_time;
                }
                KeyCode::Delete => {
                    std::sync::Arc::make_mut(&mut undo_stack).push_initial(snapshot(&buffer));
                    if buffer.delete_forward(ctrl_held) {
                        std::sync::Arc::make_mut(&mut undo_stack)
                            .push(snapshot(&buffer), current_time);
                        changed = true;
                    }
                    cursor_blink_timer = current_time;
                }
                KeyCode::ArrowLeft => {
                    buffer.move_left(ctrl_held, shift_held);
                    cursor_blink_timer = current_time;
                }
                KeyCode::ArrowRight => {
                    buffer.move_right(ctrl_held, shift_held);
                    cursor_blink_timer = current_time;
                }
                KeyCode::Home => {
                    buffer.move_home(shift_held);
                    cursor_blink_timer = current_time;
                }
                KeyCode::End => {
                    buffer.move_end(shift_held);
                    cursor_blink_timer = current_time;
                }
                KeyCode::KeyA if ctrl_held => {
                    buffer.select_all();
                    cursor_blink_timer = current_time;
                }
                KeyCode::KeyZ if ctrl_held => {
                    if let Some(restored) = std::sync::Arc::make_mut(&mut undo_stack).undo() {
                        buffer.text = restored.text.clone();
                        buffer.cursor = restored.cursor_position;
                        buffer.selection = restored.selection_start;
                        changed = true;
                        cursor_blink_timer = current_time;
                    }
                }
                KeyCode::KeyY if ctrl_held => {
                    if let Some(restored) = std::sync::Arc::make_mut(&mut undo_stack).redo() {
                        buffer.text = restored.text.clone();
                        buffer.cursor = restored.cursor_position;
                        buffer.selection = restored.selection_start;
                        changed = true;
                        cursor_blink_timer = current_time;
                    }
                }
                KeyCode::KeyC if ctrl_held => {
                    if let Some(selected) = buffer.selected_text() {
                        ui_set_clipboard_text(runtime, selected);
                    }
                }
                KeyCode::KeyX if ctrl_held => {
                    if let Some(selected) = buffer.selected_text() {
                        ui_set_clipboard_text(runtime, selected);
                        std::sync::Arc::make_mut(&mut undo_stack).push_initial(snapshot(&buffer));
                        buffer.delete_selection();
                        changed = true;
                        std::sync::Arc::make_mut(&mut undo_stack)
                            .push(snapshot(&buffer), current_time);
                    }
                }
                KeyCode::KeyV if ctrl_held => {
                    let mut paste_text = ui_read_system_clipboard(runtime);
                    if !paste_text.is_empty() {
                        if let Some(max) = max_length {
                            let removed = buffer
                                .selection_range()
                                .map(|(min, max_sel)| max_sel - min)
                                .unwrap_or(0);
                            let available = max.saturating_sub(buffer.length() - removed);
                            paste_text = paste_text.chars().take(available).collect();
                        }
                        if !paste_text.is_empty() {
                            std::sync::Arc::make_mut(&mut undo_stack)
                                .push_initial(snapshot(&buffer));
                            buffer.insert_str(&paste_text, CharStyle::default());
                            changed = true;
                            std::sync::Arc::make_mut(&mut undo_stack)
                                .push(snapshot(&buffer), current_time);
                        }
                    }
                }
                KeyCode::Escape => {
                    buffer.selection = None;
                    clear_focus = true;
                }
                KeyCode::Enter => {
                    clear_focus = true;
                    submitted = true;
                }
                _ => {}
            }
        }

        if interaction.clicked
            && let Some(rect) = super::handle_slider::node_rect(nodes, entity)
        {
            let local_x = (mouse_position.x - rect.min.x) / dpi_scale - 8.0 + scroll_offset;
            let hit_text =
                crate::commands::widgets::masked_input_display(&buffer.text, data.password);
            let new_position =
                byte_index_at_x(&mut text_state.font_engine, &hit_text, font_size, local_x);
            if shift_held {
                if buffer.selection.is_none() {
                    buffer.selection = Some(buffer.cursor);
                }
            } else {
                buffer.selection = None;
            }
            buffer.cursor = new_position;
            cursor_blink_timer = current_time;
        }

        if interaction.double_clicked {
            buffer.select_word_at_cursor();
        }
    } else {
        buffer.selection = None;
    }

    let text = buffer.text;
    let cursor_position = buffer.cursor;
    let selection_start = buffer.selection;

    if clear_focus {
        *clear_focused = true;
    }

    if changed {
        text_state.cache.set_text(
            data.text_slot,
            crate::commands::widgets::masked_input_display(&text, data.password),
        );
    }

    let cursor_visible = is_focused && ((current_time - cursor_blink_timer) % 1.0) < 0.5;
    nodes.for_entity(data.cursor_entity, |(cursor_node,)| {
        cursor_node.visible = cursor_visible
    });

    let has_selection =
        is_focused && selection_start.is_some() && selection_start != Some(cursor_position);
    nodes.for_entity(data.selection_entity, |(sel_node,)| {
        sel_node.visible = has_selection
    });

    if let Some(rect) = super::handle_slider::node_rect(nodes, entity) {
        {
            let displayed_text =
                crate::commands::widgets::masked_input_display(&text, data.password);
            let text_before_cursor: String = displayed_text.chars().take(cursor_position).collect();
            let cursor_x =
                measure_text_width(&mut text_state.font_engine, &text_before_cursor, font_size);

            let visible_width = rect.width() / dpi_scale - 16.0;
            if cursor_x - scroll_offset > visible_width {
                scroll_offset = cursor_x - visible_width;
            } else if cursor_x - scroll_offset < 0.0 {
                scroll_offset = cursor_x;
            }

            let cursor_screen_x = cursor_x - scroll_offset;

            let mut sel_start_x = 0.0f32;
            let mut sel_end_x = 0.0f32;
            if has_selection {
                let sel_start = selection_start.unwrap();
                let sel_min = sel_start.min(cursor_position);
                let sel_max = sel_start.max(cursor_position);
                let text_before_sel: String = displayed_text.chars().take(sel_min).collect();
                let text_to_sel_end: String = displayed_text.chars().take(sel_max).collect();
                sel_start_x =
                    measure_text_width(&mut text_state.font_engine, &text_before_sel, font_size)
                        - scroll_offset;
                sel_end_x =
                    measure_text_width(&mut text_state.font_engine, &text_to_sel_end, font_size)
                        - scroll_offset;
            }

            nodes.for_entity(data.cursor_entity, |(cursor_node,)| {
                if let Some(crate::components::layout_types::UiLayoutType::Window(window)) =
                    cursor_node.base_layout.as_mut()
                {
                    window.position =
                        crate::data::units::Ab(Vec2::new(8.0 + cursor_screen_x, 4.0)).into();
                }
            });

            if has_selection {
                nodes.for_entity(data.selection_entity, |(sel_node,)| {
                    if let Some(crate::components::layout_types::UiLayoutType::Window(window)) =
                        sel_node.base_layout.as_mut()
                    {
                        window.position =
                            crate::data::units::Ab(Vec2::new(8.0 + sel_start_x, 4.0)).into();
                        window.size = crate::data::units::Ab(Vec2::new(
                            sel_end_x - sel_start_x,
                            rect.height() / dpi_scale - 8.0,
                        ))
                        .into();
                    }
                });
            }
        }
    }

    if changed {
        frame
            .events
            .push(crate::resources::UiEvent::TextInputChanged {
                entity,
                text: text.clone(),
            });
        validate_widget_text(interactions, entity, &text);
    }
    if submitted {
        frame
            .events
            .push(crate::resources::UiEvent::TextInputSubmitted {
                entity,
                text: text.clone(),
            });
    }
    let text_is_empty = text.is_empty();
    let mut placeholder_entity = None;
    inputs.for_entity(entity, |(widget_data,)| {
        widget_data.text = text;
        widget_data.cursor_position = cursor_position;
        widget_data.selection_start = selection_start;
        widget_data.changed = changed;
        widget_data.cursor_blink_timer = cursor_blink_timer;
        widget_data.scroll_offset = scroll_offset;
        widget_data.undo_stack = undo_stack;
        placeholder_entity = widget_data.placeholder_entity;
    });
    if let Some(placeholder) = placeholder_entity {
        nodes.for_entity(placeholder, |(node,)| node.visible = text_is_empty);
    }
}

fn snapshot(buffer: &EditBuffer) -> TextSnapshot {
    TextSnapshot {
        text: buffer.text.clone(),
        cursor_position: buffer.cursor,
        selection_start: buffer.selection,
    }
}

pub fn ui_selectable_label_system(
    theme: Res<crate::resources::ThemeState>,
    groups: Res<crate::resources::RetainedUiGroups>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut labels: UiSelectableLabels<'_, '_>,
    mut paint: super::handle_tree::UiRowPaint<'_, '_>,
    mut interactions: super::handle_scroll::UiInteractions<'_, '_>,
) {
    use crate::data::state::UiStateTrait as _;

    let accent = theme.active_theme().accent_color;
    let selected_bg = nalgebra_glm::Vec4::new(accent.x, accent.y, accent.z, 0.3);

    let set_paint = |paint_query: &mut super::handle_tree::UiRowPaint<'_, '_>,
                     entity: nightshade::ecs::Entity,
                     selected: bool| {
        paint_query.for_entity(entity, |(color, weights)| {
            color.colors[crate::data::state::UiSelected::INDEX] = selected.then_some(selected_bg);
            weights.weights[crate::data::state::UiSelected::INDEX] =
                if selected { 1.0 } else { 0.0 };
        });
    };

    let mut pending = Vec::new();
    labels.for_each_mut(|entity, (data,)| {
        data.changed = false;
        pending.push((entity, data.selected, data.group_id));
    });

    for (entity, selected, group_id) in pending {
        if !super::handle_slider::row_interaction(&mut interactions, entity).clicked {
            continue;
        }
        let new_selected = !selected;

        if let Some(group_id) = group_id
            && new_selected
        {
            let siblings = groups
                .selectable_labels
                .get(&group_id)
                .cloned()
                .unwrap_or_default();
            for other in siblings {
                if other == entity {
                    continue;
                }
                let mut was_selected = false;
                labels.for_entity(other, |(data,)| was_selected = data.selected);
                if !was_selected {
                    continue;
                }
                set_paint(&mut paint, other, false);
                labels.for_entity(other, |(data,)| {
                    data.selected = false;
                    data.changed = true;
                });
            }
        }

        set_paint(&mut paint, entity, new_selected);
        labels.for_entity(entity, |(data,)| {
            data.selected = new_selected;
            data.changed = true;
        });
        frame
            .events
            .push(crate::resources::UiEvent::SelectableLabelClicked {
                entity,
                selected: new_selected,
            });
    }
}

pub fn ui_text_input_system(
    theme: Res<crate::resources::ThemeState>,
    timing: Res<crate::resources::RetainedUiTiming>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    input: Res<nightshade_platform::contract::input::Input>,
    window: Res<nightshade_platform::contract::window::Window>,
    mut runtime: ResMut<crate::resources::RetainedUiRuntime>,
    mut interaction_state: ResMut<crate::resources::RetainedUiInteraction>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    mut inputs: UiTextInputs<'_, '_>,
    mut nodes: super::handle_scroll::UiNodes<'_, '_>,
    mut interactions: super::handle_scroll::UiInteractions<'_, '_>,
) {
    let font_size = theme.active_theme().font_size;
    let mut clear_focused = false;
    let context = super::TextEditContext {
        focused_entity: interaction_state.focused_entity,
        frame_chars: &mirror.frame_chars,
        frame_keys: &mirror.frame_keys,
        ctrl_held: mirror.ctrl_held,
        shift_held: mirror.shift_held,
        mouse_position: input.mouse.position,
        current_time: timing.current_time,
        dpi_scale: window.cached_scale_factor,
    };

    let mut pending = Vec::new();
    inputs.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        let interaction = super::handle_slider::row_interaction(&mut interactions, entity);
        handle_text_input(
            &mut runtime,
            &mut frame,
            &mut text_state,
            &mut inputs,
            &mut nodes,
            &mut interactions,
            font_size,
            &mut clear_focused,
            entity,
            &interaction,
            &data,
            &context,
        );
    }

    if clear_focused {
        interaction_state.focused_entity = None;
    }
}
