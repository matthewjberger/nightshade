use nightshade::prelude::{Entity, vec2};
use nightshade_text::contract::data::text_data::FontKind;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::prelude::*;

use crate::commands::ink::BookInk;
use crate::data::book::{Span, SpanStyle};

#[derive(Clone, Copy)]
pub struct ProseStyle {
    pub size: f32,
    pub leading: f32,
    pub dim: bool,
}

impl ProseStyle {
    pub fn new(size: f32, leading: f32) -> Self {
        Self {
            size,
            leading,
            dim: false,
        }
    }

    pub fn dimmed(mut self) -> Self {
        self.dim = true;
        self
    }

    pub fn line_height(&self) -> f32 {
        (self.size * self.leading).round()
    }
}

pub fn add_prose(tree: &mut UiTreeBuilder, spans: &[Span], style: ProseStyle) -> Entity {
    let row = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)))
        .auto_size(AutoSizeMode::Height)
        .flow(FlowDirection::Horizontal, 0.0, 0.0)
        .flow_wrap()
        .entity();

    tree.in_parent(row, |tree| {
        for span in spans {
            add_words(tree, span, style);
        }
    });
    row
}

pub fn add_line(tree: &mut UiTreeBuilder, text: &str, style: ProseStyle) -> Entity {
    let node = tree
        .add_node()
        .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, style.line_height())))
        .with_text(text, style.size)
        .text_left();
    if style.dim {
        node.ink_dim()
    } else {
        node.ink()
    }
    .entity()
}

pub fn measure(tree: &mut UiTreeBuilder, text: &str, size: f32, kind: FontKind) -> f32 {
    let state = tree.text_state_mut();
    measure_text_width_with_kind(&mut state.font_engine, text, size, kind)
}

fn space_width(tree: &mut UiTreeBuilder, size: f32) -> f32 {
    let apart = measure(tree, "l l", size, FontKind::Default);
    let together = measure(tree, "ll", size, FontKind::Default);
    (apart - together).max(1.0)
}

pub fn set_font_kind(tree: &mut UiTreeBuilder, node: Entity, kind: FontKind) {
    tree.edit::<UiNodeContent>(node, move |content| {
        if let UiNodeContent::Text { font_kind, .. } = content {
            *font_kind = kind;
        }
    });
}

struct WordStyle {
    size: f32,
    height: f32,
    dim: bool,
}

fn add_words(tree: &mut UiTreeBuilder, span: &Span, style: ProseStyle) {
    let word_style = WordStyle {
        size: span_size(span.style, style.size),
        height: style.line_height(),
        dim: style.dim,
    };
    let kind = span_font(span.style);

    if span.text.trim().is_empty() {
        let width = space_width(tree, style.size) * span.text.chars().count() as f32;
        tree.add_node()
            .flow_child(Ab(vec2(width, word_style.height)));
        return;
    }

    let mut pending = String::new();
    for chunk in span.text.split_inclusive(' ') {
        pending.push_str(chunk);
        if pending.trim().is_empty() {
            continue;
        }
        let node = add_word(tree, &pending, &word_style, span.target.as_deref());
        if kind != FontKind::Default {
            set_font_kind(tree, node, kind);
        }
        pending.clear();
    }
}

fn add_word(
    tree: &mut UiTreeBuilder,
    word: &str,
    style: &WordStyle,
    target: Option<&str>,
) -> Entity {
    let node = tree
        .add_node()
        .flow_child(Ab(vec2(0.0, style.height)))
        .auto_size(AutoSizeMode::Width)
        .with_text(word, style.size)
        .text_left();
    let mut builder = if style.dim {
        node.ink_dim()
    } else {
        node.ink()
    };
    if target.is_some() {
        builder = builder
            .fg_hover(ThemeColor::TextAccent)
            .with_interaction()
            .with_transition::<UiHover>(12.0, 8.0)
            .with_cursor_icon(nightshade_platform::CursorIcon::Pointer);
    }
    let node = builder.entity();

    if let Some(target) = target {
        tree.commands().set(
            node,
            crate::components::PageLink {
                target: target.to_string(),
            },
        );
    }

    if target.is_none() {
        tree.edit::<UiLayoutNode>(node, |layout| {
            layout.pointer_events = false;
        });
    }
    node
}

fn span_size(style: SpanStyle, size: f32) -> f32 {
    match style {
        SpanStyle::Code => (size * 0.92).round(),
        SpanStyle::Strong => (size * 1.02).round(),
        _ => size,
    }
}

fn span_font(style: SpanStyle) -> FontKind {
    match style {
        SpanStyle::Code => FontKind::Mono,
        _ => FontKind::Default,
    }
}
