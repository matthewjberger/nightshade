use crate::resources::{DialogueCard, DialogueStyle};
use nightshade::prelude::*;
use nightshade_ui::prelude::*;

pub fn build(tree: &mut UiTreeBuilder, style: &DialogueStyle, card: &mut DialogueCard) {
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 100.0)) + Ab(vec2(0.0, -style.up)),
            Ab(vec2(style.width, style.height)),
            Anchor::BottomCenter,
        )
        .with_rect(10.0, 1.0, style.border)
        .color_raw::<UiBase>(style.background)
        .entity();

    tree.in_parent(panel, |tree| {
        card.speaker = Some(
            tree.add_node()
                .window(
                    Ab(vec2(28.0, 20.0)),
                    Ab(vec2(style.width - 56.0, style.name_size + 8.0)),
                    Anchor::TopLeft,
                )
                .with_text("", style.name_size)
                .text_left()
                .color_raw::<UiBase>(style.name_tint)
                .entity(),
        );
        card.line = Some(
            tree.add_node()
                .window(
                    Ab(vec2(28.0, 60.0)),
                    Ab(vec2(style.width - 56.0, style.line_size + 8.0)),
                    Anchor::TopLeft,
                )
                .with_text("", style.line_size)
                .text_left()
                .color_raw::<UiBase>(style.line_tint)
                .entity(),
        );
    });

    card.card = Some(panel);
}
