use crate::theme;
use nalgebra_glm::{vec2, vec4};
use nightshade::prelude::*;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::prelude::*;

pub fn build_crosshair(tree: &mut UiTreeBuilder<'_, '_>) {
    tree.add_node()
        .window(Rl(vec2(50.0, 50.0)), Ab(vec2(2.0, 16.0)), Anchor::Center)
        .with_rect(0.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(theme::CROSSHAIR_COLOR);
    tree.add_node()
        .window(Rl(vec2(50.0, 50.0)), Ab(vec2(16.0, 2.0)), Anchor::Center)
        .with_rect(0.0, 0.0, vec4(0.0, 0.0, 0.0, 0.0))
        .color_raw::<UiBase>(theme::CROSSHAIR_COLOR);
}
