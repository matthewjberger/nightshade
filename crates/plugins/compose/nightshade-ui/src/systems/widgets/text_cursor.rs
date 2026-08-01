use crate::data::units::measure_text_width;
use nalgebra_glm::Vec2;
use nightshade_text::contract::data::font_engine::FontEngine;

pub type UiValidations<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (&'item mut crate::components::UiNodeInteraction,),
>;

pub(super) fn validate_widget_text(
    validations: &mut super::handle_scroll::UiInteractions<'_, '_>,
    entity: nightshade::ecs::Entity,
    text: &str,
) {
    let mut rules: Vec<crate::components::ValidationRule> = Vec::new();
    validations.for_entity(entity, |(interaction,)| {
        rules = interaction.validation_rules.clone()
    });

    if rules.is_empty() {
        return;
    }

    let mut error: Option<String> = None;
    for rule in &rules {
        if let Err(message) = rule.validate(text) {
            error = Some(message);
            break;
        }
    }

    validations.for_entity(entity, |(interaction,)| {
        interaction.error_text = error.clone();
        interaction.tooltip_text = error;
    });
}

pub fn byte_index_at_x(
    font_engine: &mut FontEngine,
    text: &str,
    font_size: f32,
    target_x: f32,
) -> usize {
    if text.is_empty() {
        return 0;
    }
    let line_height = font_size * 1.2;
    let buffer = font_engine.shape_buffer(
        text,
        font_size,
        line_height,
        None,
        nightshade_text::contract::data::text_data::FontKind::Default,
    );
    if let Some(cursor) = buffer.hit(target_x.max(0.0), 0.0) {
        let byte_index = cursor.index;
        text[..byte_index.min(text.len())].chars().count()
    } else {
        text.chars().count()
    }
}

pub fn prev_word_boundary(text: &str, position: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    if position == 0 {
        return 0;
    }
    let mut index = position - 1;
    while index > 0 && chars[index].is_whitespace() {
        index -= 1;
    }
    while index > 0 && !chars[index - 1].is_whitespace() {
        index -= 1;
    }
    index
}

pub fn next_word_boundary(text: &str, position: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    if position >= len {
        return len;
    }
    let mut index = position;
    while index < len && !chars[index].is_whitespace() {
        index += 1;
    }
    while index < len && chars[index].is_whitespace() {
        index += 1;
    }
    index
}

pub(super) fn line_col_from_char_position(text: &str, position: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    for (index, character) in text.chars().enumerate() {
        if index == position {
            return (line, col);
        }
        if character == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

pub(super) fn char_position_from_line_col(
    text: &str,
    target_line: usize,
    target_col: usize,
) -> usize {
    let mut line = 0;
    let mut col = 0;
    for (index, character) in text.chars().enumerate() {
        if line == target_line && col == target_col {
            return index;
        }
        if line > target_line {
            return index.saturating_sub(1);
        }
        if character == '\n' {
            if line == target_line {
                return index;
            }
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    text.chars().count()
}

pub(super) fn line_count(text: &str) -> usize {
    text.chars().filter(|character| *character == '\n').count() + 1
}

pub(super) fn line_start_char_index(text: &str, target_line: usize) -> usize {
    let mut line = 0;
    for (index, character) in text.chars().enumerate() {
        if line == target_line {
            return index;
        }
        if character == '\n' {
            line += 1;
        }
    }
    text.chars().count()
}

pub(super) fn line_text(text: &str, target_line: usize) -> &str {
    text.split('\n').nth(target_line).unwrap_or("")
}

pub(super) struct TextCursorLayout<'a> {
    pub(super) cursor_entity: nightshade::ecs::Entity,
    pub(super) selection_pool: &'a [nightshade::ecs::Entity],
    pub(super) visible_rows: usize,
    pub(super) line_height: f32,
}

pub(super) struct TextCursorState {
    pub(super) cursor_position: usize,
    pub(super) selection_start: Option<usize>,
    pub(super) is_focused: bool,
    pub(super) current_time: f64,
    pub(super) cursor_blink_timer: f64,
    pub(super) scroll_offset_y: f32,
}

pub(super) fn place_text_cursor(
    nodes: &mut super::handle_scroll::UiNodes<'_, '_>,
    text_state: &mut nightshade_text::contract::resources::TextState,
    font_size: f32,
    entity: nightshade::ecs::Entity,
    layout: &TextCursorLayout<'_>,
    text: &str,
    state: &TextCursorState,
) -> f32 {
    let TextCursorState {
        cursor_position,
        selection_start,
        is_focused,
        current_time,
        cursor_blink_timer,
        mut scroll_offset_y,
    } = *state;
    let line_height = layout.line_height;
    let cursor_visible = is_focused && ((current_time - cursor_blink_timer) % 1.0) < 0.5;
    nodes.for_entity(layout.cursor_entity, |(cursor_node,)| {
        cursor_node.visible = cursor_visible
    });

    let (cur_line, cur_col) = line_col_from_char_position(text, cursor_position);
    let visible_rows = layout.visible_rows;
    let cursor_y = cur_line as f32 * line_height;
    let visible_height = visible_rows as f32 * line_height;
    if cursor_y - scroll_offset_y >= visible_height {
        scroll_offset_y = cursor_y - visible_height + line_height;
    } else if cursor_y < scroll_offset_y {
        scroll_offset_y = cursor_y;
    }
    let total_lines = line_count(text);
    let max_scroll = ((total_lines as f32 * line_height) - visible_height).max(0.0);
    scroll_offset_y = scroll_offset_y.clamp(0.0, max_scroll);

    let mut has_node = false;
    nodes.for_entity(entity, |_| has_node = true);
    if has_node {
        {
            let cur_line_text = line_text(text, cur_line).to_string();
            let text_before_cursor: String = cur_line_text.chars().take(cur_col).collect();
            let cursor_x =
                measure_text_width(&mut text_state.font_engine, &text_before_cursor, font_size);
            let cursor_screen_y = cur_line as f32 * line_height - scroll_offset_y;

            let has_selection =
                is_focused && selection_start.is_some() && selection_start != Some(cursor_position);

            let sel_positions: Vec<(f32, f32, f32)> = if has_selection {
                let sel_start = selection_start.unwrap();
                let sel_min = sel_start.min(cursor_position);
                let sel_max = sel_start.max(cursor_position);
                let (min_line, min_col) = line_col_from_char_position(text, sel_min);
                let (max_line, max_col) = line_col_from_char_position(text, sel_max);

                let mut positions = Vec::new();
                for line_index in min_line..=max_line {
                    let line_segment = line_text(text, line_index).to_string();
                    let start_col = if line_index == min_line { min_col } else { 0 };
                    let end_col = if line_index == max_line {
                        max_col
                    } else {
                        line_segment.chars().count()
                    };
                    let start_text: String = line_segment.chars().take(start_col).collect();
                    let end_text: String = line_segment.chars().take(end_col).collect();
                    let start_x =
                        measure_text_width(&mut text_state.font_engine, &start_text, font_size);
                    let end_x =
                        measure_text_width(&mut text_state.font_engine, &end_text, font_size);
                    let start_y = line_index as f32 * line_height - scroll_offset_y;
                    positions.push((start_x, end_x - start_x, start_y));
                }
                positions
            } else {
                Vec::new()
            };

            nodes.for_entity(layout.cursor_entity, |(cursor_node,)| {
                if let Some(crate::components::layout_types::UiLayoutType::Window(window)) =
                    cursor_node.base_layout.as_mut()
                {
                    window.position =
                        crate::data::units::Ab(Vec2::new(8.0 + cursor_x, 8.0 + cursor_screen_y))
                            .into();
                }
            });

            for (pool_index, sel_entity) in layout.selection_pool.iter().enumerate() {
                if pool_index < sel_positions.len() {
                    let (start_x, width, start_y) = sel_positions[pool_index];
                    nodes.for_entity(*sel_entity, |(sel_node,)| {
                        sel_node.visible = true;
                        if let Some(crate::components::layout_types::UiLayoutType::Window(window)) =
                            sel_node.base_layout.as_mut()
                        {
                            window.position =
                                crate::data::units::Ab(Vec2::new(8.0 + start_x, 8.0 + start_y))
                                    .into();
                            window.size =
                                crate::data::units::Ab(Vec2::new(width.max(2.0), line_height))
                                    .into();
                        }
                    });
                } else {
                    nodes.for_entity(*sel_entity, |(sel_node,)| sel_node.visible = false);
                }
            }
        }
    }
    scroll_offset_y
}
