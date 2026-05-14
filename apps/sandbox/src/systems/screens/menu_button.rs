use crate::theme::*;
use nightshade::prelude::*;

/// Builds the standard sandbox menu button: a flow-child rect with
/// hover/pressed/focused state colors, an accent side bar, and a
/// left-aligned label. Title and pause screens share this so the
/// styling lives in one place.
pub fn build(tree: &mut UiTreeBuilder, label: &str) -> Entity {
    let button = tree
        .add_node()
        .flow_child(Ab(MENU_BUTTON_SIZE))
        .with_rect(2.0, 1.0, PANEL_BORDER)
        .color_raw::<UiBase>(PANEL_BG)
        .color_raw::<UiHover>(PANEL_HOVER)
        .color_raw::<UiPressed>(PANEL_PRESSED)
        .color_raw::<UiFocused>(ACCENT_DIM)
        .with_transition::<UiHover>(14.0, 8.0)
        .with_transition::<UiPressed>(20.0, 12.0)
        .with_transition::<UiFocused>(14.0, 8.0)
        .with_interaction()
        .with_cursor_icon(winit::window::CursorIcon::Pointer)
        .entity();
    tree.in_parent(button, |tree| {
        tree.add_node()
            .window(
                Ab(vec2(20.0, 0.0)) + Rl(vec2(0.0, 50.0)),
                Ab(vec2(3.0, MENU_BUTTON_HEIGHT - 16.0)),
                Anchor::CenterLeft,
            )
            .with_rect(0.0, 0.0, TRANSPARENT)
            .color_raw::<UiBase>(ACCENT)
            .color_raw::<UiFocused>(ACCENT_HOT)
            .with_transition::<UiFocused>(14.0, 8.0)
            .entity();
        tree.add_node()
            .window(
                Ab(vec2(40.0, 0.0)) + Rl(vec2(0.0, 50.0)),
                Ab(vec2(MENU_BUTTON_SIZE.x - 60.0, MENU_BUTTON_HEIGHT)),
                Anchor::CenterLeft,
            )
            .with_text(label, 20.0)
            .text_left()
            .color_raw::<UiBase>(TEXT_COLOR)
            .color_raw::<UiFocused>(WHITE)
            .with_transition::<UiFocused>(14.0, 8.0)
            .entity();
    });
    button
}
