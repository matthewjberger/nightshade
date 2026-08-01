use nightshade::ecs::system_param::{Query, Res, ResMut};
use nightshade_platform::KeyCode;

use super::handle_scroll::{UiInteractions, UiNodes};
use super::handle_slider::{RowInteraction, row_interaction};
use super::text_cursor::{byte_index_at_x, line_count, line_start_char_index, line_text};
use super::text_edit::EditBuffer;
use crate::components::{CharStyle, TextSnapshot};

use crate::prelude::*;

pub type UiTextAreas<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiTextAreaData,)>;

fn handle_text_area(
    runtime: &mut crate::resources::RetainedUiRuntime,
    frame: &mut crate::resources::RetainedUiFrame,
    text_state: &mut nightshade_text::contract::resources::TextState,
    text_areas: &mut UiTextAreas<'_, '_>,
    nodes: &mut UiNodes<'_, '_>,
    entity: nightshade::ecs::Entity,
    interaction: &RowInteraction,
    data: &crate::components::UiTextAreaData,
    context: &super::TextEditContext<'_>,
    font_size: f32,
    clear_focused: &mut bool,
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
    let mut scroll_offset_y = data.scroll_offset_y;
    let mut clear_focus = false;
    let mut submitted = false;
    let line_height = data.line_height;
    let mut undo_stack = data.undo_stack.clone();
    let max_length = data.max_length;

    let mut buffer = EditBuffer::new(
        data.text.clone(),
        data.cursor_position,
        data.selection_start,
        true,
    );

    if is_focused {
        let mut needs_snapshot = false;
        for character in frame_chars {
            if *character >= ' ' {
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
                    undo_stack.push_initial(snapshot(&buffer));
                    needs_snapshot = true;
                }
                buffer.insert_char(*character, CharStyle::default());
                changed = true;
                cursor_blink_timer = current_time;
            }
        }

        if needs_snapshot {
            undo_stack.push(snapshot(&buffer), current_time);
        }

        for (key, is_pressed) in frame_keys {
            if !is_pressed {
                continue;
            }
            match key {
                KeyCode::Backspace => {
                    undo_stack.push_initial(snapshot(&buffer));
                    if buffer.backspace(ctrl_held) {
                        undo_stack.push(snapshot(&buffer), current_time);
                        changed = true;
                    }
                    cursor_blink_timer = current_time;
                }
                KeyCode::Delete => {
                    undo_stack.push_initial(snapshot(&buffer));
                    if buffer.delete_forward(ctrl_held) {
                        undo_stack.push(snapshot(&buffer), current_time);
                        changed = true;
                    }
                    cursor_blink_timer = current_time;
                }
                KeyCode::Enter => {
                    if ctrl_held {
                        clear_focus = true;
                        submitted = true;
                    } else {
                        undo_stack.push_initial(snapshot(&buffer));
                        buffer.insert_newline(CharStyle::default());
                        undo_stack.push(snapshot(&buffer), current_time);
                        changed = true;
                        cursor_blink_timer = current_time;
                    }
                }
                KeyCode::ArrowLeft => {
                    buffer.move_left(ctrl_held, shift_held);
                    cursor_blink_timer = current_time;
                }
                KeyCode::ArrowRight => {
                    buffer.move_right(ctrl_held, shift_held);
                    cursor_blink_timer = current_time;
                }
                KeyCode::ArrowUp => {
                    buffer.move_up(shift_held);
                    cursor_blink_timer = current_time;
                }
                KeyCode::ArrowDown => {
                    buffer.move_down(shift_held);
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
                    if let Some(restored) = undo_stack.undo() {
                        buffer.text = restored.text.clone();
                        buffer.cursor = restored.cursor_position;
                        buffer.selection = restored.selection_start;
                        changed = true;
                        cursor_blink_timer = current_time;
                    }
                }
                KeyCode::KeyY if ctrl_held => {
                    if let Some(restored) = undo_stack.redo() {
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
                        undo_stack.push_initial(snapshot(&buffer));
                        buffer.delete_selection();
                        changed = true;
                        undo_stack.push(snapshot(&buffer), current_time);
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
                            undo_stack.push_initial(snapshot(&buffer));
                            buffer.insert_str(&paste_text, CharStyle::default());
                            changed = true;
                            undo_stack.push(snapshot(&buffer), current_time);
                        }
                    }
                }
                KeyCode::Escape => {
                    buffer.selection = None;
                    clear_focus = true;
                }
                _ => {}
            }
        }

        if interaction.clicked
            && let Some(rect) = super::handle_slider::node_rect(nodes, entity)
        {
            let local_y = (mouse_position.y - rect.min.y) / dpi_scale - 8.0 + scroll_offset_y;
            let clicked_line = (local_y / line_height).max(0.0) as usize;
            let total_lines = line_count(&buffer.text);
            let target_line = clicked_line.min(total_lines.saturating_sub(1));
            let target_line_text = line_text(&buffer.text, target_line).to_string();
            let local_x = (mouse_position.x - rect.min.x) / dpi_scale - 8.0;
            let column = byte_index_at_x(
                &mut text_state.font_engine,
                &target_line_text,
                font_size,
                local_x,
            );
            let line_start = line_start_char_index(&buffer.text, target_line);
            if shift_held {
                if buffer.selection.is_none() {
                    buffer.selection = Some(buffer.cursor);
                }
            } else {
                buffer.selection = None;
            }
            buffer.cursor = line_start + column;
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
        text_state.cache.set_text(data.text_slot, &text);
    }

    scroll_offset_y = super::text_cursor::place_text_cursor(
        nodes,
        text_state,
        font_size,
        entity,
        &super::text_cursor::TextCursorLayout {
            cursor_entity: data.cursor_entity,
            selection_pool: &data.selection_pool,
            visible_rows: data.visible_rows,
            line_height,
        },
        &text,
        &super::text_cursor::TextCursorState {
            cursor_position,
            selection_start,
            is_focused,
            current_time,
            cursor_blink_timer,
            scroll_offset_y,
        },
    );

    if changed {
        frame
            .events
            .push(crate::resources::UiEvent::TextAreaChanged {
                entity,
                text: text.clone(),
            });
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
    text_areas.for_entity(entity, |(widget_data,)| {
        widget_data.text = text;
        widget_data.cursor_position = cursor_position;
        widget_data.selection_start = selection_start;
        widget_data.changed = changed;
        widget_data.cursor_blink_timer = cursor_blink_timer;
        widget_data.scroll_offset_y = scroll_offset_y;
        widget_data.undo_stack = undo_stack;
        placeholder_entity = widget_data.placeholder_entity;
    });
    if let Some(placeholder) = placeholder_entity {
        nodes.for_entity(placeholder, |(node,)| node.visible = text_is_empty);
    }
}

pub fn ui_text_area_system(
    theme: Res<crate::resources::ThemeState>,
    timing: Res<crate::resources::RetainedUiTiming>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    input: Res<nightshade_platform::contract::input::Input>,
    window: Res<nightshade_platform::contract::window::Window>,
    mut runtime: ResMut<crate::resources::RetainedUiRuntime>,
    mut interaction_state: ResMut<crate::resources::RetainedUiInteraction>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    mut text_areas: UiTextAreas<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
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
    text_areas.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        let interaction = row_interaction(&mut interactions, entity);
        handle_text_area(
            &mut runtime,
            &mut frame,
            &mut text_state,
            &mut text_areas,
            &mut nodes,
            entity,
            &interaction,
            &data,
            &context,
            font_size,
            &mut clear_focused,
        );
    }

    if clear_focused {
        interaction_state.focused_entity = None;
    }
}

fn snapshot(buffer: &EditBuffer) -> TextSnapshot {
    TextSnapshot {
        text: buffer.text.clone(),
        cursor_position: buffer.cursor,
        selection_start: buffer.selection,
    }
}
