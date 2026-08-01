use crate::resources::HudBuild;
use crate::resources::RatePanel;
use crate::theme;
use nalgebra_glm::vec2;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::components::TextOverflow;
use nightshade_ui::components::layout_types::FlowDirection;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_menu::prelude::add_frame;

pub fn build_readouts(tree: &mut UiTreeBuilder<'_, '_>, hud: &mut HudBuild<'_>) {
    let status_card = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(50.0, 100.0))
                + Ab(vec2(
                    0.0,
                    -theme::HOTBAR_BOTTOM_MARGIN - theme::SLOT_SIZE - 12.0,
                )),
            Ab(vec2(theme::STATUS_WIDTH, theme::STATUS_HEIGHT)),
            Anchor::BottomCenter,
        )
        .flow(FlowDirection::Vertical, 5.0, 0.0)
        .entity();
    let frame_layer = hud.art.frame;
    hud.hotbar.status = tree.in_parent(status_card, |tree| {
        add_frame(tree, hud.style, frame_layer);
        tree.add_node()
            .size_px(theme::STATUS_WIDTH - 10.0, theme::STATUS_HEIGHT - 10.0)
            .with_text("", theme::SMALL_SIZE)
            .with_text_overflow(TextOverflow::Ellipsis)
            .text_center()
            .color_raw::<UiBase>(theme::TEXT_COLOR)
            .entity()
    });
}

pub fn build_rate(tree: &mut UiTreeBuilder<'_, '_>, hud: &mut HudBuild<'_>) {
    hud.rate.panel = tree
        .card_node(10.0, 2.0, theme::PANEL_BORDER)
        .color_raw::<UiBase>(theme::PANEL_COLOR)
        .window(
            Rl(vec2(100.0, 100.0)) + Ab(vec2(-14.0, -14.0)),
            Ab(vec2(theme::RATE_CARD_WIDTH, theme::STATUS_HEIGHT)),
            Anchor::BottomRight,
        )
        .flow(FlowDirection::Vertical, 5.0, 0.0)
        .with_visible(false)
        .entity();
    let frame_layer = hud.art.frame;
    tree.in_parent(hud.rate.panel, |tree| {
        add_frame(tree, hud.style, frame_layer);
        hud.rate.label = tree
            .add_node()
            .size_px(theme::RATE_CARD_WIDTH - 10.0, theme::STATUS_HEIGHT - 10.0)
            .with_text("", theme::BODY_SIZE)
            .text_center()
            .color_raw::<UiBase>(theme::TEXT_COLOR)
            .entity();
    });
}

pub fn show_the_rate(
    rate_panel: &mut RatePanel,
    show_rate: bool,
    time: &Time,
    gpu_milliseconds: f32,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
) {
    if rate_panel.shown != Some(show_rate) {
        rate_panel.shown = Some(show_rate);
        ui_set_visible(dirty, commands, rate_panel.panel, show_rate);
    }
    if !show_rate {
        return;
    }
    let rate = time.frames_per_second;
    let text = format!("{rate:.0} fps   gpu {gpu_milliseconds:.1} ms");
    if text == rate_panel.shown_text {
        return;
    }
    rate_panel.shown_text = text;
    ui_set_text(commands, rate_panel.label, &rate_panel.shown_text);
    let color = if rate >= theme::RATE_GOOD {
        theme::RATE_GOOD_COLOR
    } else if rate >= theme::RATE_FAIR {
        theme::RATE_FAIR_COLOR
    } else {
        theme::RATE_POOR_COLOR
    };
    if rate_panel.shown_color != Some(color) {
        rate_panel.shown_color = Some(color);
        ui_set_color::<UiBase>(commands, rate_panel.label, color);
    }
}
