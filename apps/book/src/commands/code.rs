use nightshade::prelude::{Entity, vec2};
use nightshade_text::contract::data::text_data::FontKind;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::prelude::*;

use crate::commands::ink::{BookInk, token_color};
use crate::commands::prose::{measure, set_font_kind};
use crate::data::style::{
    CARD_HEADER_HEIGHT, CARD_PADDING, CARD_RADIUS, CODE_LEADING, CODE_SIZE, HAIRLINE, LABEL_LINE,
    LABEL_SIZE, PAGE_MAX_WIDTH, SPACE_2, SPACE_3,
};
use crate::queries::code::tokenize;
use crate::queries::prose::type_scale;

const SMALLEST_CODE_SIZE: f32 = 11.0;

pub fn add_code_card(tree: &mut UiTreeBuilder, language: &str, lines: &[String]) -> Entity {
    let theme = tree.active_theme().clone();
    let room = PAGE_MAX_WIDTH - CARD_PADDING * 2.0;
    let asked = CODE_SIZE * type_scale(tree);
    let widest = lines
        .iter()
        .map(|line| measure(tree, line, asked, FontKind::Mono))
        .fold(1.0_f32, f32::max);
    let size = if widest > room {
        (asked * room / widest).floor().max(SMALLEST_CODE_SIZE)
    } else {
        asked
    };

    let line_height = (size * CODE_LEADING).round();
    let body_height = lines.len() as f32 * line_height + CARD_PADDING * 2.0 + HAIRLINE;

    let card = tree
        .add_node()
        .flow_child(
            Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, body_height + CARD_HEADER_HEIGHT + HAIRLINE * 2.0)),
        )
        .with_rect(CARD_RADIUS, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .with_theme_color::<UiBase>(ThemeColor::InputBackground)
        .flow(FlowDirection::Vertical, HAIRLINE, 0.0)
        .with_clip()
        .entity();

    tree.in_parent(card, |tree| {
        add_header(tree, language, lines.len());
        add_body(tree, lines, language, size, line_height);
    });
    card
}

fn add_header(tree: &mut UiTreeBuilder, language: &str, count: usize) {
    let header = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, CARD_HEADER_HEIGHT)))
        .rect(0.0)
        .fg(ThemeColor::PanelHeader)
        .flow(FlowDirection::Horizontal, SPACE_3, SPACE_2)
        .align_cross(FlowAlignment::Center)
        .entity();

    tree.in_parent(header, |tree| {
        let name = if language.is_empty() {
            "text".to_string()
        } else {
            language.to_string()
        };
        let label = tree
            .add_node()
            .flow_child(Ab(vec2(0.0, LABEL_LINE)))
            .auto_size(AutoSizeMode::Width)
            .with_text(&name, LABEL_SIZE)
            .text_left()
            .ink_dim()
            .entity();
        set_font_kind(tree, label, FontKind::Mono);

        tree.add_node()
            .flow_child(Ab(vec2(0.0, LABEL_LINE)))
            .flex_grow(1.0);

        let count = if count == 1 {
            "1 line".to_string()
        } else {
            format!("{count} lines")
        };
        tree.add_node()
            .flow_child(Ab(vec2(0.0, LABEL_LINE)))
            .auto_size(AutoSizeMode::Width)
            .with_text(&count, LABEL_SIZE)
            .text_left()
            .ink_dim();
    });

    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, HAIRLINE)))
        .rect(0.0)
        .fg(ThemeColor::Border);
}

fn add_body(
    tree: &mut UiTreeBuilder,
    lines: &[String],
    language: &str,
    size: f32,
    line_height: f32,
) {
    let body = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)))
        .flex_grow(1.0)
        .flow(FlowDirection::Vertical, CARD_PADDING, 0.0)
        .with_clip()
        .entity();

    tree.in_parent(body, |tree| {
        for line in lines {
            let row = tree
                .add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, line_height)))
                .flow(FlowDirection::Horizontal, 0.0, 0.0)
                .entity();
            tree.in_parent(row, |tree| {
                add_code_line(tree, line, language, size, line_height)
            });
        }
    });
}

pub fn add_code_line(
    tree: &mut UiTreeBuilder,
    line: &str,
    language: &str,
    size: f32,
    line_height: f32,
) {
    for (text, token) in tokenize(line, language) {
        if text.is_empty() {
            continue;
        }
        let width = measure(tree, &text, size, FontKind::Mono);
        let node = tree
            .add_node()
            .flow_child(Ab(vec2(width, line_height)))
            .with_text(&text, size)
            .text_left()
            .ink_colored(token_color(token))
            .entity();
        set_font_kind(tree, node, FontKind::Mono);
    }
}
