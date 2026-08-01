use crate::resources::HudBuild;
use crate::theme;
use nalgebra_glm::vec2;
use nightshade::prelude::*;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::add_frame;

pub fn build_prompt(tree: &mut UiTreeBuilder<'_, '_>, hud: &mut HudBuild<'_>) {
    hud.prompt.panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 50.0)) + Ab(vec2(0.0, 60.0)),
            Ab(vec2(380.0, 44.0)),
            Anchor::Center,
        )
        .flow(FlowDirection::Vertical, 8.0, 0.0)
        .with_visible(false)
        .entity();
    let frame_layer = hud.art.frame;
    tree.in_parent(hud.prompt.panel, |tree| {
        add_frame(tree, hud.style, frame_layer);
        hud.prompt.label = tree
            .add_node()
            .fill_width()
            .auto_height()
            .with_text("", theme::BODY_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::ACCENT_COLOR)
            .entity();
    });
}
