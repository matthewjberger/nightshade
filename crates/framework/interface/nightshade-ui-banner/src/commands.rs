use crate::data::BannerStyle;
use crate::resources::{Banner, BannerHandles};
use nalgebra_glm::Vec4;
use nightshade::prelude::vec2;
use nightshade_text::contract::data::text_data::{TextAlignment, VerticalAlignment};
use nightshade_ui::prelude::*;

pub fn add_banner(tree: &mut UiTreeBuilder, style: &BannerStyle) -> BannerHandles {
    let panel = tree
        .add_node()
        .window(
            Rl(vec2(50.0, 0.0)) + Ab(vec2(0.0, style.margin)),
            Ab(vec2(style.width, style.height)),
            Anchor::TopCenter,
        )
        .with_rect(style.corner_radius, style.border_width, style.border)
        .color_raw::<UiBase>(style.background)
        .without_pointer_events()
        .entity();
    let mut text = nightshade::ecs::Entity::default();
    tree.in_parent(panel, |tree| {
        let mut node = tree
            .add_node()
            .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
            .with_text("", style.size)
            .with_text_alignment(TextAlignment::Center, VerticalAlignment::Middle)
            .color_raw::<UiBase>(style.color)
            .without_pointer_events();
        if style.outline_width > 0.0 {
            node = node.with_text_outline(style.outline, style.outline_width);
        }
        text = node.entity();
    });
    BannerHandles { panel, text }
}

pub fn say(banner: &mut Banner, line: String) {
    banner.line = line;
}

pub fn tint(banner: &mut Banner, color: Vec4) {
    banner.color = color;
}

pub fn show(banner: &mut Banner, shown: bool) {
    banner.shown = shown;
}
