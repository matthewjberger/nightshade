use crate::resources::{StoryCard, StoryStyle};
use nightshade::prelude::vec2;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;

pub fn build(tree: &mut UiTreeBuilder, style: &StoryStyle, card: &mut StoryCard) {
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 50.0)),
            Ab(vec2(style.width, crate::queries::height(style))),
            Anchor::Center,
        )
        .with_rect(style.corner_radius, style.border_width, style.border)
        .color_raw::<UiBase>(style.background)
        .flow(FlowDirection::Vertical, style.padding, style.spacing)
        .entity();

    let mut lines = Vec::with_capacity(style.lines);
    let mut links = Vec::with_capacity(style.links);
    let mut input = None;
    tree.in_parent(panel, |tree| {
        for _ in 0..style.lines {
            lines.push(
                tree.add_node()
                    .flow_child(Rl(vec2(100.0, 0.0)) + Ab(vec2(0.0, style.line_size * 1.35)))
                    .with_text("", style.line_size)
                    .with_text_alignment(TextAlignment::Left, VerticalAlignment::Middle)
                    .color_raw::<UiBase>(style.line_tint)
                    .entity(),
            );
        }
        for _ in 0..style.links {
            links.push(tree.add_button(""));
        }
        input = Some(tree.add_text_input("What do you do?"));
    });

    card.panel = Some(panel);
    card.shown = vec![String::new(); lines.len()];
    card.labels = vec![String::new(); links.len()];
    card.lit = vec![true; links.len()];
    card.asking = true;
    card.lines = lines;
    card.links = links;
    card.input = input;
}
