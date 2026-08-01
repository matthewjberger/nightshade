use nightshade::prelude::{Entity, vec2, vec4};
use nightshade_text::contract::data::text_data::FontKind;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::data::icons::material;
use nightshade_ui::prelude::*;

use crate::commands::code::add_code_line;
use crate::commands::ink::BookInk;
use crate::commands::prose::{measure, set_font_kind};
use crate::data::source::{SourceFile, files_of, tree_of};
use crate::data::style::{
    CARD_PADDING, CODE_LEADING, CODE_SIZE, HAIRLINE, LABEL_LINE, LABEL_SIZE, SOURCE_GUTTER,
    SOURCE_INDENT, SOURCE_ROW, SOURCE_SMALLEST, SOURCE_TREE_WIDTH, SPACE_2, SPACE_3,
};
use crate::queries::code::language_of;

pub struct SourceTree {
    pub rows: Vec<Entity>,
    pub row_files: Vec<usize>,
}

pub fn build_source(tree: &mut UiTreeBuilder, demo: &str, chosen: usize, room: f32) -> SourceTree {
    let files = files_of(demo);
    let mut built = SourceTree {
        rows: Vec::new(),
        row_files: Vec::new(),
    };

    let column = tree
        .add_node()
        .boundary(
            Ab(vec2(0.0, 0.0)),
            Rl(vec2(0.0, 100.0)) + Ab(vec2(SOURCE_TREE_WIDTH, 0.0)),
        )
        .rect(0.0)
        .fg(ThemeColor::PanelHeader)
        .flow(FlowDirection::Vertical, 0.0, 0.0)
        .entity();
    tree.in_parent(column, |tree| {
        built = add_file_list(tree, &files, chosen);
    });

    tree.add_node()
        .boundary(
            Ab(vec2(SOURCE_TREE_WIDTH, 0.0)),
            Rl(vec2(0.0, 100.0)) + Ab(vec2(SOURCE_TREE_WIDTH + HAIRLINE, 0.0)),
        )
        .rect(0.0)
        .fg(ThemeColor::Border);

    let reading = tree
        .add_node()
        .boundary(
            Ab(vec2(SOURCE_TREE_WIDTH + HAIRLINE, 0.0)),
            Rl(vec2(100.0, 100.0)),
        )
        .flow(FlowDirection::Vertical, 0.0, 0.0)
        .entity();
    tree.in_parent(reading, |tree| match files.get(chosen) {
        Some(file) => add_file(tree, file, room),
        None => {
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, SOURCE_ROW)))
                .with_text("This demo carries no source.", LABEL_SIZE)
                .text_center()
                .ink_dim();
        }
    });

    built
}

fn add_file_list(
    tree: &mut UiTreeBuilder,
    files: &[&'static SourceFile],
    chosen: usize,
) -> SourceTree {
    let mut rows = Vec::new();
    let mut row_files = Vec::new();

    let scroll = tree.add_scroll_area_fill(SPACE_2, 0.0);
    let Some(host) = scroll_content(tree, scroll) else {
        return SourceTree { rows, row_files };
    };

    tree.in_parent(host, |tree| {
        for line in tree_of(files) {
            let indent = SPACE_2 + line.depth as f32 * SOURCE_INDENT;
            let Some(index) = line.file else {
                add_folder_row(tree, &line.name, indent);
                continue;
            };

            let row = add_file_row(tree, &line.name, indent, line.own, index == chosen);
            rows.push(row);
            row_files.push(index);
        }
    });

    SourceTree { rows, row_files }
}

fn add_indent(tree: &mut UiTreeBuilder, indent: f32) {
    tree.add_node().flow_child(Ab(vec2(indent, SOURCE_ROW)));
}

fn add_folder_row(tree: &mut UiTreeBuilder, name: &str, indent: f32) {
    let row = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, SOURCE_ROW)))
        .flow(FlowDirection::Horizontal, 0.0, SPACE_2)
        .align_cross(FlowAlignment::Center)
        .entity();

    tree.in_parent(row, |tree| {
        add_indent(tree, indent);
        tree.add_node()
            .flow_child(Ab(vec2(13.0, SOURCE_ROW)))
            .with_icon(material::FOLDER, 12.0)
            .text_center()
            .ink_dim();
        tree.add_node()
            .flow_child(Ab(vec2(0.0, SOURCE_ROW)))
            .flex_grow(1.0)
            .with_text(name, LABEL_SIZE)
            .with_text_overflow(nightshade_ui::components::TextOverflow::Ellipsis)
            .text_left()
            .ink_dim();
    });
}

fn add_file_row(
    tree: &mut UiTreeBuilder,
    name: &str,
    indent: f32,
    own: bool,
    open: bool,
) -> Entity {
    let accent = tree.active_theme().accent_color;
    let fill = if open {
        vec4(accent.x, accent.y, accent.z, 0.22)
    } else {
        vec4(0.0, 0.0, 0.0, 0.0)
    };

    let row = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, SOURCE_ROW)))
        .with_rect(4.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(fill)
        .with_theme_color::<UiHover>(ThemeColor::BackgroundHover)
        .with_interaction()
        .with_transition::<UiHover>(12.0, 9.0)
        .with_cursor_icon(nightshade_platform::CursorIcon::Pointer)
        .flow(FlowDirection::Horizontal, 0.0, SPACE_2)
        .align_cross(FlowAlignment::Center)
        .entity();

    tree.in_parent(row, |tree| {
        add_indent(tree, indent);
        let icon = tree
            .add_node()
            .flow_child(Ab(vec2(13.0, SOURCE_ROW)))
            .with_icon(material::INSERT_DRIVE_FILE, 12.0)
            .text_center();
        if own {
            icon.ink();
        } else {
            icon.ink_dim();
        }

        let label = tree
            .add_node()
            .flow_child(Ab(vec2(0.0, SOURCE_ROW)))
            .flex_grow(1.0)
            .with_text(name, LABEL_SIZE)
            .with_text_overflow(nightshade_ui::components::TextOverflow::Ellipsis)
            .text_left();
        let label = if own { label.ink() } else { label.ink_dim() };
        let label = label.entity();
        set_font_kind(tree, label, FontKind::Mono);
    });

    row
}

fn add_file(tree: &mut UiTreeBuilder, file: &SourceFile, room: f32) {
    let lines: Vec<&str> = file.text.lines().collect();
    let widest = lines
        .iter()
        .map(|line| measure(tree, line, CODE_SIZE, FontKind::Mono))
        .fold(1.0_f32, f32::max);
    let size = if widest > room {
        (CODE_SIZE * room / widest).floor().max(SOURCE_SMALLEST)
    } else {
        CODE_SIZE
    };
    let line_height = (size * CODE_LEADING).round();
    let language = language_of(file.path);

    add_file_header(tree, file, lines.len());

    let body = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)))
        .flex_grow(1.0)
        .entity();
    tree.in_parent(body, |tree| {
        let scroll = tree.add_scroll_area_fill(CARD_PADDING, 0.0);
        let Some(host) = scroll_content(tree, scroll) else {
            return;
        };
        tree.in_parent(host, |tree| {
            for (number, line) in lines.iter().enumerate() {
                add_line(tree, number + 1, line, language, size, line_height);
            }
        });
    });
}

fn add_file_header(tree: &mut UiTreeBuilder, file: &SourceFile, count: usize) {
    let header = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, LABEL_LINE + SPACE_3 * 2.0)))
        .rect(0.0)
        .fg(ThemeColor::PanelHeader)
        .flow(FlowDirection::Horizontal, SPACE_3, SPACE_2)
        .align_cross(FlowAlignment::Center)
        .entity();

    tree.in_parent(header, |tree| {
        let path = tree
            .add_node()
            .flow_child(Ab(vec2(0.0, LABEL_LINE)))
            .auto_size(AutoSizeMode::Width)
            .with_text(file.path, LABEL_SIZE)
            .text_left()
            .ink()
            .entity();
        set_font_kind(tree, path, FontKind::Mono);

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

fn add_line(
    tree: &mut UiTreeBuilder,
    number: usize,
    line: &str,
    language: &str,
    size: f32,
    line_height: f32,
) {
    let row = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, line_height)))
        .flow(FlowDirection::Horizontal, 0.0, 0.0)
        .entity();

    tree.in_parent(row, |tree| {
        let count = number.to_string();
        let gutter = tree
            .add_node()
            .flow_child(Ab(vec2(SOURCE_GUTTER, line_height)))
            .with_text(&count, size)
            .text_right()
            .ink_dim()
            .entity();
        set_font_kind(tree, gutter, FontKind::Mono);

        tree.add_node().flow_child(Ab(vec2(SPACE_3, line_height)));
        add_code_line(tree, line, language, size, line_height);
    });
}

fn scroll_content(tree: &mut UiTreeBuilder, scroll_area: Entity) -> Option<Entity> {
    tree.commands()
        .staged::<UiScrollAreaData>(scroll_area)
        .map(|data| data.content_entity)
}
