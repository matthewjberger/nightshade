use nalgebra_glm;
use nalgebra_glm::{Vec2, Vec4};
use nightshade::ecs::system_param::{Query, Res, ResMut};
use nightshade_platform::KeyCode;

use super::handle_scroll::{UiInteractions, UiNodes};
use super::handle_slider::{RowInteraction, row_interaction};
use super::text_cursor::{byte_index_at_x, line_count, line_start_char_index, line_text};
use super::text_edit::EditBuffer;
use crate::components::{CharStyle, RichTextSnapshot};

pub type UiRichTextEditors<'world, 'item> =
    Query<'world, (&'item mut crate::components::UiRichTextEditorData,)>;

pub(super) struct RichTextEditorContext<'a> {
    pub(super) entity: nightshade::ecs::Entity,
    pub(super) interaction: &'a RowInteraction,
    pub(super) data: &'a crate::components::UiRichTextEditorData,
    pub(super) focused_entity: Option<nightshade::ecs::Entity>,
    pub(super) frame_chars: &'a [char],
    pub(super) frame_keys: &'a [(KeyCode, bool)],
    pub(super) ctrl_held: bool,
    pub(super) shift_held: bool,
    pub(super) mouse_position: Vec2,
    pub(super) current_time: f64,
    pub(super) dpi_scale: f32,
}

fn handle_rich_text_editor(
    frame: &mut crate::resources::RetainedUiFrame,
    text_cache: &mut crate::resources::RetainedUiTextCache,
    text_state: &mut nightshade_text::contract::resources::TextState,
    editors: &mut UiRichTextEditors<'_, '_>,
    nodes: &mut UiNodes<'_, '_>,
    font_size: f32,
    clear_focused: &mut bool,
    context: RichTextEditorContext<'_>,
) {
    let entity = context.entity;
    let is_focused = context.focused_entity == Some(entity);
    let data = context.data;
    let ctrl_held = context.ctrl_held;
    let shift_held = context.shift_held;
    let current_time = context.current_time;
    let mut current_style = data.current_style.clone();
    let mut changed = false;
    let mut cursor_blink_timer = data.cursor_blink_timer;
    let mut scroll_offset_y = data.scroll_offset_y;
    let mut clear_focus = false;
    let mut submitted = false;
    let line_height = data.line_height;
    let mut undo_stack = data.undo_stack.clone();

    let mut buffer = EditBuffer::styled(
        data.text.clone(),
        data.char_styles.clone(),
        data.cursor_position,
        data.selection_start,
        true,
    );

    if is_focused {
        let mut needs_snapshot = false;
        for character in context.frame_chars {
            if *character >= ' ' {
                if !needs_snapshot {
                    undo_stack.push_initial(snapshot(&buffer));
                    needs_snapshot = true;
                }
                buffer.insert_char(*character, current_style.clone());
                changed = true;
                cursor_blink_timer = current_time;
            }
        }

        if needs_snapshot {
            undo_stack.push(snapshot(&buffer), current_time);
        }

        for (key, is_pressed) in context.frame_keys {
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
                        buffer.insert_newline(current_style.clone());
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
                KeyCode::KeyB if ctrl_held => {
                    toggle_selection_style(&mut buffer, &mut current_style, |style| {
                        &mut style.bold
                    });
                    changed = true;
                }
                KeyCode::KeyI if ctrl_held => {
                    toggle_selection_style(&mut buffer, &mut current_style, |style| {
                        &mut style.italic
                    });
                    changed = true;
                }
                KeyCode::KeyU if ctrl_held => {
                    toggle_selection_style(&mut buffer, &mut current_style, |style| {
                        &mut style.underline
                    });
                    changed = true;
                }
                KeyCode::KeyZ if ctrl_held => {
                    if let Some(restored) = undo_stack.undo() {
                        buffer.text = restored.text.clone();
                        buffer.styles = restored.char_styles.clone();
                        buffer.cursor = restored.cursor_position;
                        buffer.selection = restored.selection_start;
                        changed = true;
                        cursor_blink_timer = current_time;
                    }
                }
                KeyCode::KeyY if ctrl_held => {
                    if let Some(restored) = undo_stack.redo() {
                        buffer.text = restored.text.clone();
                        buffer.styles = restored.char_styles.clone();
                        buffer.cursor = restored.cursor_position;
                        buffer.selection = restored.selection_start;
                        changed = true;
                        cursor_blink_timer = current_time;
                    }
                }
                KeyCode::Escape => {
                    buffer.selection = None;
                    clear_focus = true;
                }
                _ => {}
            }
        }

        if context.interaction.clicked
            && let Some(rect) = super::handle_slider::node_rect(nodes, entity)
        {
            let local_y =
                (context.mouse_position.y - rect.min.y) / context.dpi_scale - 8.0 + scroll_offset_y;
            let clicked_line = (local_y / line_height).max(0.0) as usize;
            let total_lines = line_count(&buffer.text);
            let target_line = clicked_line.min(total_lines.saturating_sub(1));
            let target_line_text = line_text(&buffer.text, target_line).to_string();
            let local_x = (context.mouse_position.x - rect.min.x) / context.dpi_scale - 8.0;
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

        if context.interaction.double_clicked {
            buffer.select_word_at_cursor();
        }
    } else {
        buffer.selection = None;
    }

    let text = buffer.text;
    let char_styles = buffer.styles;
    let cursor_position = buffer.cursor;
    let selection_start = buffer.selection;

    if clear_focus {
        *clear_focused = true;
    }

    if changed {
        text_state.cache.set_text(data.text_slot, &text);

        let has_colors = char_styles.iter().any(|style| style.color.is_some());
        if has_colors {
            let colors: Vec<Option<Vec4>> = char_styles.iter().map(|style| style.color).collect();
            text_cache.character_colors.insert(data.text_slot, colors);
        } else {
            text_cache.character_colors.remove(&data.text_slot);
        }
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
            .push(crate::resources::UiEvent::RichTextEditorChanged {
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
    let mut placeholder_update = None;
    editors.for_entity(entity, |(widget_data,)| {
        let text_is_empty = text.is_empty();
        widget_data.text = text;
        widget_data.char_styles = char_styles;
        widget_data.current_style = current_style;
        widget_data.cursor_position = cursor_position;
        widget_data.selection_start = selection_start;
        widget_data.changed = changed;
        widget_data.cursor_blink_timer = cursor_blink_timer;
        widget_data.scroll_offset_y = scroll_offset_y;
        widget_data.undo_stack = undo_stack;
        placeholder_update = widget_data
            .placeholder_entity
            .filter(|_| changed)
            .map(|placeholder| (placeholder, text_is_empty));
    });

    if let Some((placeholder, text_is_empty)) = placeholder_update {
        nodes.for_entity(placeholder, |(node,)| node.visible = text_is_empty);
    }
}

pub fn ui_rich_text_editor_system(
    theme: Res<crate::resources::ThemeState>,
    timing: Res<crate::resources::RetainedUiTiming>,
    mirror: Res<crate::resources::RetainedUiInputMirror>,
    input: Res<nightshade_platform::contract::input::Input>,
    window: Res<nightshade_platform::contract::window::Window>,
    mut interaction_state: ResMut<crate::resources::RetainedUiInteraction>,
    mut frame: ResMut<crate::resources::RetainedUiFrame>,
    mut text_cache: ResMut<crate::resources::RetainedUiTextCache>,
    mut text_state: ResMut<nightshade_text::contract::resources::TextState>,
    mut editors: UiRichTextEditors<'_, '_>,
    mut nodes: UiNodes<'_, '_>,
    mut interactions: UiInteractions<'_, '_>,
) {
    let font_size = theme.active_theme().font_size;
    let focused_entity = interaction_state.focused_entity;
    let mut clear_focused = false;

    let mut pending = Vec::new();
    editors.for_each_mut(|entity, (data,)| pending.push((entity, data.clone())));

    for (entity, data) in pending {
        let interaction = row_interaction(&mut interactions, entity);
        handle_rich_text_editor(
            &mut frame,
            &mut text_cache,
            &mut text_state,
            &mut editors,
            &mut nodes,
            font_size,
            &mut clear_focused,
            RichTextEditorContext {
                entity,
                interaction: &interaction,
                data: &data,
                focused_entity,
                frame_chars: &mirror.frame_chars,
                frame_keys: &mirror.frame_keys,
                ctrl_held: mirror.ctrl_held,
                shift_held: mirror.shift_held,
                mouse_position: input.mouse.position,
                current_time: timing.current_time,
                dpi_scale: window.cached_scale_factor,
            },
        );
    }

    if clear_focused {
        interaction_state.focused_entity = None;
    }
}

fn snapshot(buffer: &EditBuffer) -> RichTextSnapshot {
    RichTextSnapshot {
        text: buffer.text.clone(),
        char_styles: buffer.styles.clone(),
        cursor_position: buffer.cursor,
        selection_start: buffer.selection,
    }
}

fn toggle_selection_style(
    buffer: &mut EditBuffer,
    current_style: &mut CharStyle,
    accessor: fn(&mut CharStyle) -> &mut bool,
) {
    if let Some((start, end)) = buffer.selection_range() {
        let all_set = (start..end).all(|index| {
            buffer
                .styles
                .get(index)
                .map(|style| *accessor(&mut style.clone()))
                .unwrap_or(false)
        });
        let new_value = !all_set;
        for index in start..end {
            if index < buffer.styles.len() {
                *accessor(&mut buffer.styles[index]) = new_value;
            }
        }
        *accessor(current_style) = new_value;
    } else {
        let field = accessor(current_style);
        *field = !*field;
    }
}
