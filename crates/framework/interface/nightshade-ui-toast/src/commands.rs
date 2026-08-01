use crate::resources::{ToastCard, ToastStyle};
use nightshade::prelude::*;
use nightshade_ui::prelude::*;

pub fn build(tree: &mut UiTreeBuilder, style: &ToastStyle, card: &mut ToastCard) {
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, style.down)),
            Ab(vec2(style.width, style.height)),
            Anchor::TopCenter,
        )
        .with_rect(8.0, 1.0, style.border)
        .color_raw::<UiBase>(style.background)
        .entity();

    tree.in_parent(panel, |tree| {
        card.line = Some(
            tree.add_node()
                .window(
                    Rl(vec2(50.0, 50.0)),
                    Ab(vec2(style.width - 32.0, style.text_size + 8.0)),
                    Anchor::Center,
                )
                .with_text("", style.text_size)
                .color_raw::<UiBase>(style.text)
                .entity(),
        );
    });

    card.card = Some(panel);
}
