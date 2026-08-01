use nalgebra_glm;
use nalgebra_glm::Vec2;
use nightshade_platform::KeyCode;

use nightshade::ecs::system_param::{Query, Res, ResMut};

use super::handle_scroll::{UiInteractions, UiNodes};
use super::handle_slider::{RowInteraction, node_rect, row_interaction};
use super::text_cursor::byte_index_at_x;
use super::text_edit::EditBuffer;
use crate::components::{CharStyle, TextSnapshot};
use crate::data::units::measure_text_width;

use crate::prelude::*;

pub type UiDragValues<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiDragValueData,)>;

fn handle_drag_value(
    runtime: &mut crate::resources::RetainedUiRuntime,
    frame: &mut crate::resources::RetainedUiFrame,
    text_state: &mut nightshade_text::contract::resources::TextState,
    values: &mut UiDragValues<'_, '_>,
    nodes: &mut UiNodes<'_, '_>,
    interactions: &mut UiInteractions<'_, '_>,
    font_size: f32,
    focus: &mut FocusChange,
    entity: nightshade::ecs::Entity,
    interaction: &RowInteraction,
    data: &crate::components::UiDragValueData,
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
    let mut value = data.value;
    let mut changed = false;
    let mut editing = data.editing;
    let mut edit_text = data.edit_text.clone();
    let mut cursor_position = data.cursor_position;
    let mut selection_start = data.selection_start;
    let mut cursor_blink_timer = data.cursor_blink_timer;
    let mut scroll_offset = data.scroll_offset;
    let mut drag_start_value = data.drag_start_value;
    let mut undo_stack = data.undo_stack.clone();
    let mut clear_focus = false;

    if editing && is_focused {
        let mut buffer =
            EditBuffer::new(edit_text.clone(), cursor_position, selection_start, false);
        let mut needs_snapshot = false;
        for character in frame_chars {
            if *character >= ' '
                && (character.is_ascii_digit() || *character == '.' || *character == '-')
            {
                if !needs_snapshot {
                    undo_stack.push_initial(snapshot(&buffer));
                    needs_snapshot = true;
                }
                buffer.insert_char(*character, CharStyle::default());
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
                    buffer.backspace(ctrl_held);
                    undo_stack.push(snapshot(&buffer), current_time);
                    cursor_blink_timer = current_time;
                }
                KeyCode::Delete => {
                    undo_stack.push_initial(snapshot(&buffer));
                    buffer.delete_forward(ctrl_held);
                    undo_stack.push(snapshot(&buffer), current_time);
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
                    if let Some(restored) = undo_stack.undo() {
                        buffer.text = restored.text.clone();
                        buffer.cursor = restored.cursor_position;
                        buffer.selection = restored.selection_start;
                        cursor_blink_timer = current_time;
                    }
                }
                KeyCode::KeyY if ctrl_held => {
                    if let Some(restored) = undo_stack.redo() {
                        buffer.text = restored.text.clone();
                        buffer.cursor = restored.cursor_position;
                        buffer.selection = restored.selection_start;
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
                        undo_stack.push(snapshot(&buffer), current_time);
                    }
                }
                KeyCode::KeyV if ctrl_held => {
                    let paste_text = ui_read_system_clipboard(runtime);
                    if !paste_text.is_empty() {
                        undo_stack.push_initial(snapshot(&buffer));
                        buffer.insert_str(&paste_text, CharStyle::default());
                        undo_stack.push(snapshot(&buffer), current_time);
                    }
                }
                KeyCode::Escape => {
                    editing = false;
                    buffer.selection = None;
                    clear_focus = true;
                }
                KeyCode::Enter => {
                    if let Ok(parsed) = buffer.text.parse::<f32>() {
                        value = parsed.clamp(data.min, data.max);
                        changed = true;
                    }
                    editing = false;
                    buffer.selection = None;
                    clear_focus = true;
                }
                _ => {}
            }
        }

        if interaction.clicked
            && let Some(rect) = node_rect(nodes, entity)
        {
            let local_x = (mouse_position.x - rect.min.x) / dpi_scale - 8.0 + scroll_offset;
            let new_position = byte_index_at_x(
                &mut text_state.font_engine,
                &buffer.text,
                font_size,
                local_x,
            );
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

        edit_text = buffer.text;
        cursor_position = buffer.cursor;
        selection_start = buffer.selection;

        text_state.cache.set_text(data.text_slot, &edit_text);
    } else if editing && !is_focused {
        if let Ok(parsed) = edit_text.parse::<f32>() {
            value = parsed.clamp(data.min, data.max);
            changed = true;
        }
        editing = false;
        selection_start = None;
    } else {
        if interaction.dragging
            && let Some(drag_start) = interaction.drag_start
        {
            let delta_x = mouse_position.x - drag_start.x;
            let pixel_threshold = 3.0;
            if delta_x.abs() > pixel_threshold {
                let new_value = (drag_start_value + (delta_x / dpi_scale) * data.speed)
                    .clamp(data.min, data.max);
                if (new_value - value).abs() > f32::EPSILON {
                    value = new_value;
                    changed = true;
                }
            }
        }

        if interaction.clicked && !interaction.dragging {
            editing = true;
            edit_text = format!("{:.prec$}", data.value, prec = data.precision);
            cursor_position = edit_text.chars().count();
            selection_start = Some(0);
            cursor_blink_timer = current_time;
            *focus = FocusChange::Take;
        }

        if interaction.pressed && !interaction.dragging && interaction.drag_start.is_some() {
            drag_start_value = value;
        }
    }

    if clear_focus {
        *focus = FocusChange::Clear;
    }

    if !editing {
        let display = format!(
            "{}{:.prec$}{}",
            data.prefix,
            value,
            data.suffix,
            prec = data.precision
        );
        text_state.cache.set_text(data.text_slot, &display);
        scroll_offset = 0.0;
    }

    let cursor_visible = editing && is_focused && ((current_time - cursor_blink_timer) % 1.0) < 0.5;
    nodes.for_entity(data.cursor_entity, |(cursor_node,)| {
        cursor_node.visible = cursor_visible
    });

    let has_selection = editing
        && is_focused
        && selection_start.is_some()
        && selection_start != Some(cursor_position);
    nodes.for_entity(data.selection_entity, |(sel_node,)| {
        sel_node.visible = has_selection
    });

    if editing && let Some(rect) = node_rect(nodes, entity) {
        let text_before_cursor: String = edit_text.chars().take(cursor_position).collect();
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
            let text_before_sel: String = edit_text.chars().take(sel_min).collect();
            let text_to_sel_end: String = edit_text.chars().take(sel_max).collect();
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

    if let Some(up) = data.up_entity
        && row_interaction(interactions, up).clicked
    {
        value = (value + data.step).min(data.max);
        changed = true;
    }
    if let Some(down) = data.down_entity
        && row_interaction(interactions, down).clicked
    {
        value = (value - data.step).max(data.min);
        changed = true;
    }

    let cursor_icon = if editing {
        nightshade_platform::CursorIcon::Text
    } else {
        nightshade_platform::CursorIcon::EwResize
    };
    interactions.for_entity(entity, |(row,)| row.cursor_icon = Some(cursor_icon));

    values.for_entity(entity, |(widget_data,)| {
        widget_data.value = value;
        widget_data.changed = changed;
        widget_data.editing = editing;
        widget_data.edit_text = edit_text;
        widget_data.cursor_position = cursor_position;
        widget_data.selection_start = selection_start;
        widget_data.cursor_blink_timer = cursor_blink_timer;
        widget_data.scroll_offset = scroll_offset;
        widget_data.drag_start_value = drag_start_value;
        widget_data.undo_stack = undo_stack;
    });

    if changed {
        frame
            .events
            .push(crate::resources::UiEvent::DragValueChanged { entity, value });
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum FocusChange {
    #[default]
    None,
    Take,
    Clear,
}

pub fn ui_drag_value_system(
    theme: Res<crate::resources::ThemeState>,
    timing: Res<crate::resources::RetainedUiTiming>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    input: Res<nightshade_platform::contract::input::Input>,
    window: Res<nightshade_platform::contract::window::Window>,
    mut interaction_state: ResMut<crate::resources::RetainedUiInteraction>,
    mut runtime: ResMut<crate::resources::RetainedUiRuntime>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    mut values: UiDragValues<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
) {
    let font_size = theme.active_theme().font_size;
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
    values.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    let mut focus = FocusChange::None;
    let mut focus_entity = None;
    for (entity, data) in pending {
        let interaction = row_interaction(&mut interactions, entity);
        let mut row_focus = FocusChange::None;
        handle_drag_value(
            &mut runtime,
            &mut frame,
            &mut text_state,
            &mut values,
            &mut nodes,
            &mut interactions,
            font_size,
            &mut row_focus,
            entity,
            &interaction,
            &data,
            &context,
        );
        if row_focus != FocusChange::None {
            focus = row_focus;
            focus_entity = Some(entity);
        }
    }

    match focus {
        FocusChange::None => {}
        FocusChange::Take => interaction_state.focused_entity = focus_entity,
        FocusChange::Clear => interaction_state.focused_entity = None,
    }
}

fn snapshot(buffer: &EditBuffer) -> TextSnapshot {
    TextSnapshot {
        text: buffer.text.clone(),
        cursor_position: buffer.cursor,
        selection_start: buffer.selection,
    }
}
