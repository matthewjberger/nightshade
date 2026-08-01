use nightshade::prelude::{Entity, vec2, vec4};
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::prelude::*;

use crate::commands::code::add_code_card;
use crate::commands::figure::{Figure, add_figure};
use crate::commands::ink::BookInk;
use crate::commands::picture::Pictures;
use crate::commands::prose::{ProseStyle, add_line, add_prose};
use crate::data::book::{Block, Chapter, ListItem, Span, SpanStyle};
use crate::data::style::{
    BODY_LEADING, BODY_SIZE, CAPTION_LEADING, CAPTION_SIZE, CARD_RADIUS, CHECK_BOX_SIZE,
    GAP_AFTER_HEADING, GAP_BEFORE_HEADING_1, GAP_BEFORE_HEADING_2, GAP_BEFORE_HEADING_3, GAP_CARD,
    GAP_LIST_ITEM, GAP_PARAGRAPH, HAIRLINE, HEADING_LEADING, HEADING_SIZES, LIST_MARKER_WIDTH,
    PAGE_MAX_WIDTH, PAGE_PADDING, QUOTE_BAR_WIDTH, SPACE_2, SPACE_3, SPACE_4, SPACE_5, SPACE_6,
    SPACE_7,
};
use crate::queries::prose::type_scale;
use crate::resources::PageStage;

pub fn build_blocks(
    tree: &mut UiTreeBuilder,
    chapter: &Chapter,
    page_stage: &mut PageStage,
    pictures: &mut Pictures,
) {
    let mut figures = 0;
    let mut previous: Option<&Block> = None;

    for block in &chapter.blocks {
        if let Some(previous) = previous {
            let gap = gap_between(previous, block);
            if gap > 0.0 {
                tree.add_spacing(gap);
            }
        }
        build_block(tree, block, page_stage, &mut figures, pictures);
        previous = Some(block);
    }

    if !chapter.footnotes.is_empty() {
        tree.add_spacing(SPACE_6);
        build_footnotes(tree, chapter);
    }

    tree.add_spacing(SPACE_7);
}

fn build_block(
    tree: &mut UiTreeBuilder,
    block: &Block,
    page_stage: &mut PageStage,
    figures: &mut usize,
    pictures: &mut Pictures,
) {
    match block {
        Block::Heading {
            level,
            spans,
            anchor,
        } => {
            let entity = build_heading(tree, *level, spans);
            page_stage.headings.push((anchor.clone(), entity));
        }
        Block::Paragraph { spans } => {
            add_prose(tree, spans, body_style(tree));
        }
        Block::List {
            ordered,
            start,
            items,
        } => build_list(tree, *ordered, *start, items),
        Block::Code { language, lines } => {
            add_code_card(tree, language, lines);
        }
        Block::Quote { spans } => build_quote(tree, spans),
        Block::Table { headers, rows } => build_table(tree, headers, rows),
        Block::Image { path, caption } => build_image(tree, pictures, path, caption),
        Block::Rule => build_rule(tree),
        Block::Stage {
            demo,
            title,
            caption,
        } => {
            *figures += 1;
            build_stage(tree, demo, title, caption, *figures, page_stage);
        }
    }
}

fn gap_between(previous: &Block, next: &Block) -> f32 {
    if let Block::Heading { level, .. } = next {
        return match level {
            1 => GAP_BEFORE_HEADING_1,
            2 => GAP_BEFORE_HEADING_2,
            _ => GAP_BEFORE_HEADING_3,
        };
    }
    if matches!(previous, Block::Heading { .. }) {
        return GAP_AFTER_HEADING;
    }
    if matches!(
        previous,
        Block::Code { .. } | Block::Stage { .. } | Block::Table { .. } | Block::Image { .. }
    ) || matches!(
        next,
        Block::Code { .. } | Block::Stage { .. } | Block::Table { .. } | Block::Image { .. }
    ) {
        return GAP_CARD;
    }
    if matches!(previous, Block::List { .. }) && matches!(next, Block::List { .. }) {
        return GAP_LIST_ITEM;
    }
    GAP_PARAGRAPH
}

pub fn body_style(tree: &UiTreeBuilder) -> ProseStyle {
    ProseStyle::new(BODY_SIZE * type_scale(tree), BODY_LEADING)
}

fn caption_style(tree: &UiTreeBuilder) -> ProseStyle {
    ProseStyle::new(CAPTION_SIZE * type_scale(tree), CAPTION_LEADING).dimmed()
}

fn build_heading(tree: &mut UiTreeBuilder, level: u8, spans: &[Span]) -> Entity {
    let index = (level.clamp(1, 6) as usize) - 1;
    let size = HEADING_SIZES[index] * type_scale(tree);
    let style = ProseStyle::new(size, HEADING_LEADING);
    let entity = add_prose(tree, spans, style);

    if level == 1 {
        tree.add_spacing(SPACE_3);
        tree.add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 1.0)))
            .rect(0.0)
            .fg(ThemeColor::Border);
    }
    entity
}

fn build_list(tree: &mut UiTreeBuilder, ordered: bool, start: u64, items: &[ListItem]) {
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            tree.add_spacing(GAP_LIST_ITEM);
        }

        let indent = item.depth as f32 * SPACE_5;
        let row = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)))
            .auto_size(AutoSizeMode::Height)
            .flow(FlowDirection::Horizontal, 0.0, 0.0)
            .entity();

        tree.in_parent(row, |tree| {
            match item.done {
                Some(done) => add_check_box(tree, indent, done),
                None => {
                    let marker = marker_text(ordered, start, index);
                    let line_height = body_style(tree).line_height();
                    let marker_size = BODY_SIZE * type_scale(tree);
                    tree.add_node()
                        .flow_child(Ab(vec2(indent + LIST_MARKER_WIDTH, line_height)))
                        .with_text(&marker, marker_size)
                        .text_right()
                        .ink_dim();
                }
            }

            let text = tree
                .add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(-(indent + LIST_MARKER_WIDTH), 0.0)))
                .auto_size(AutoSizeMode::Height)
                .flow(FlowDirection::Vertical, 0.0, 0.0)
                .entity();
            tree.in_parent(text, |tree| {
                add_prose(tree, &item.spans, body_style(tree));
            });
        });
    }
}

fn marker_text(ordered: bool, start: u64, index: usize) -> String {
    if ordered {
        return format!("{}.  ", start + index as u64);
    }
    "\u{2022}  ".to_string()
}

fn add_check_box(tree: &mut UiTreeBuilder, indent: f32, done: bool) {
    let theme = tree.active_theme().clone();
    let line_height = body_style(tree).line_height();
    let column = tree
        .add_node()
        .flow_child(Ab(vec2(indent + LIST_MARKER_WIDTH, line_height)))
        .flow_with_alignment(
            FlowDirection::Horizontal,
            0.0,
            0.0,
            FlowAlignment::End,
            FlowAlignment::Center,
        )
        .entity();

    tree.in_parent(column, |tree| {
        let box_node = tree
            .add_node()
            .flow_child(Ab(vec2(CHECK_BOX_SIZE, CHECK_BOX_SIZE)))
            .with_rect(3.0, theme.border_width, theme.border_color)
            .with_theme_border_color(if done {
                ThemeColor::Accent
            } else {
                ThemeColor::Border
            })
            .entity();
        tree.edit::<UiThemeBinding>(box_node, move |binding| {
            binding.color_roles[UiBase::INDEX] = done.then_some(ThemeColor::Accent);
        });
        tree.add_node().flow_child(Ab(vec2(SPACE_2, 0.0)));
    });
}

fn build_quote(tree: &mut UiTreeBuilder, spans: &[Span]) {
    let row = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)))
        .auto_size(AutoSizeMode::Height)
        .flow(FlowDirection::Horizontal, 0.0, SPACE_4)
        .entity();

    tree.in_parent(row, |tree| {
        tree.add_node()
            .flow_child(Rl(vec2(0.0, 100.0)) + Ab(vec2(QUOTE_BAR_WIDTH, 0.0)))
            .rect(2.0)
            .fg(ThemeColor::Accent);

        let text = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(-(QUOTE_BAR_WIDTH + SPACE_4), 0.0)))
            .auto_size(AutoSizeMode::Height)
            .flow(FlowDirection::Vertical, 0.0, 0.0)
            .entity();
        tree.in_parent(text, |tree| {
            let style = ProseStyle::new(BODY_SIZE * type_scale(tree), BODY_LEADING).dimmed();
            add_prose(tree, spans, style);
        });
    });
}

fn build_table(tree: &mut UiTreeBuilder, headers: &[Vec<Span>], rows: &[Vec<Vec<Span>>]) {
    let theme = tree.active_theme().clone();
    let columns = headers.len().max(1);

    let table = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)))
        .auto_size(AutoSizeMode::Height)
        .with_rect(CARD_RADIUS, theme.border_width, theme.border_color)
        .with_theme_border_color(ThemeColor::Border)
        .color_raw::<UiBase>(vec4(0.0, 0.0, 0.0, 0.0))
        .flow(FlowDirection::Vertical, 0.0, 0.0)
        .with_clip()
        .entity();

    tree.in_parent(table, |tree| {
        build_table_row(tree, headers, columns, true);
        for row in rows {
            tree.add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, HAIRLINE)))
                .rect(0.0)
                .fg(ThemeColor::Border);
            build_table_row(tree, row, columns, false);
        }
    });
}

fn build_table_row(tree: &mut UiTreeBuilder, cells: &[Vec<Span>], columns: usize, header: bool) {
    let mut node = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)))
        .auto_size(AutoSizeMode::Height)
        .flow(FlowDirection::Horizontal, 0.0, 0.0);
    if header {
        node = node.rect(0.0).fg(ThemeColor::PanelHeader);
    }
    let row = node.entity();

    let share = 100.0 / columns as f32;
    tree.in_parent(row, |tree| {
        for index in 0..columns {
            let cell = tree
                .add_node()
                .flow_child(Rl(vec2(share, 0.0)))
                .auto_size(AutoSizeMode::Height)
                .flow(FlowDirection::Vertical, SPACE_3, 0.0)
                .entity();
            tree.in_parent(cell, |tree| {
                let empty = Vec::new();
                let spans = cells.get(index).unwrap_or(&empty);
                let style = if header {
                    ProseStyle::new((CAPTION_SIZE + 1.0) * type_scale(tree), CAPTION_LEADING)
                } else {
                    ProseStyle::new((CAPTION_SIZE + 1.5) * type_scale(tree), BODY_LEADING)
                };
                add_prose(tree, spans, style);
            });
        }
    });
}

fn build_image(tree: &mut UiTreeBuilder, pictures: &mut Pictures, path: &str, caption: &str) {
    let name = crate::queries::picture::name_of(path);
    let Some(picture) = pictures.get(tree, name) else {
        add_line(
            tree,
            &format!("The book does not carry a picture called {name}."),
            caption_style(tree),
        );
        return;
    };

    let height = picture.height_at(PAGE_MAX_WIDTH - PAGE_PADDING * 2.0);
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, height)))
        .with_image_uv(picture.layer, picture.uv_min, picture.uv_max)
        .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 1.0));

    if !caption.is_empty() {
        tree.add_spacing(SPACE_2);
        add_line(tree, caption, caption_style(tree));
    }
}

fn build_rule(tree: &mut UiTreeBuilder) {
    tree.add_spacing(SPACE_2);
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 1.0)))
        .rect(0.0)
        .fg(ThemeColor::Border);
    tree.add_spacing(SPACE_2);
}

fn build_stage(
    tree: &mut UiTreeBuilder,
    demo: &str,
    title: &str,
    caption: &str,
    number: usize,
    page_stage: &mut PageStage,
) {
    let entry = crate::systems::demos::demo_entry(demo);
    let controls = entry.map(|entry| entry.controls).unwrap_or(&[]);
    let heading = if title.is_empty() {
        entry.map(|entry| entry.title).unwrap_or("")
    } else {
        title
    };

    let twice = entry.is_some_and(|entry| {
        page_stage
            .figures
            .iter()
            .any(|figure| figure.demo == entry.id)
    });

    let figure = add_figure(tree, number, heading, controls);
    name_for_the_hand(tree, number, &figure);
    match entry {
        Some(entry) if !twice => page_stage.figures.push(crate::resources::Figure {
            demo: entry.id,
            card: figure.card,
            node: figure.stage,
            controls: figure.controls,
            source_button: figure.source_button,
            expand_button: figure.expand_button,
            expand_label: figure.expand_label,
            card_height: figure.card_height,
        }),
        Some(_) => {
            tree.add_spacing(SPACE_2);
            add_line(
                tree,
                "This page already carries this demo. A page can hold several                  figures, but each of them a different one.",
                caption_style(tree),
            );
        }
        None => {
            tree.add_spacing(SPACE_2);
            add_line(
                tree,
                "This page asks for a demo the book did not compile in.",
                caption_style(tree),
            );
        }
    }

    if !caption.is_empty() {
        tree.add_spacing(SPACE_3);
        let spans = vec![Span {
            text: caption.to_string(),
            style: SpanStyle::Plain,
            target: None,
        }];
        add_prose(tree, &spans, caption_style(tree));
    }
}

fn build_footnotes(tree: &mut UiTreeBuilder, chapter: &Chapter) {
    tree.add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, 1.0)))
        .rect(0.0)
        .fg(ThemeColor::Border);
    tree.add_spacing(SPACE_4);

    for (index, (_, spans)) in chapter.footnotes.iter().enumerate() {
        if index > 0 {
            tree.add_spacing(SPACE_2);
        }
        let row = tree
            .add_node()
            .flow_child(Rl(vec2(100.0, 0.0)))
            .auto_size(AutoSizeMode::Height)
            .flow(FlowDirection::Horizontal, 0.0, 0.0)
            .entity();
        tree.in_parent(row, |tree| {
            let marker = format!("[{}]", index + 1);
            tree.add_node()
                .flow_child(Ab(vec2(LIST_MARKER_WIDTH + SPACE_2, 0.0)))
                .auto_size(AutoSizeMode::Height)
                .with_text(&marker, CAPTION_SIZE)
                .text_left()
                .ink_dim();
            let text = tree
                .add_node()
                .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(-(LIST_MARKER_WIDTH + SPACE_2), 0.0)))
                .auto_size(AutoSizeMode::Height)
                .flow(FlowDirection::Vertical, 0.0, 0.0)
                .entity();
            tree.in_parent(text, |tree| {
                add_prose(tree, spans, caption_style(tree));
            });
        });
    }
}

fn name_for_the_hand(tree: &mut UiTreeBuilder, number: usize, figure: &Figure) {
    let (accessibility, commands) = tree.accessibility_and_commands();
    ui_set_test_id(
        accessibility,
        commands,
        figure.source_button,
        &format!("figure-{number}-source"),
    );
    for (index, (_, entity, _)) in figure.controls.iter().enumerate() {
        ui_set_test_id(
            accessibility,
            commands,
            *entity,
            &format!("figure-{number}-control-{index}"),
        );
    }
}
