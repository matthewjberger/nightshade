use crate::resources::HudBuild;
use crate::theme;
use nalgebra_glm::{vec2, vec4};
use nightshade::prelude::*;
use nightshade_ui::commands::builder::UiTreeBuilder;
use nightshade_ui::prelude::*;
use nightshade_ui_menu::prelude::SlotHandles;

pub fn build_hotbar(
    tree: &mut UiTreeBuilder<'_, '_>,
    hud: &mut HudBuild<'_>,
    atlas_upload: &Option<UiImageUpload>,
) {
    let pitch = theme::SLOT_SIZE + theme::SLOT_GAP;
    let total = pitch * blocker_blocks::data::items::HOTBAR_SLOTS as f32 - theme::SLOT_GAP;
    let mut slots = [SlotHandles::default(); blocker_blocks::data::items::HOTBAR_SLOTS];
    for (index, slot) in slots.iter_mut().enumerate() {
        let left = -total * 0.5 + index as f32 * pitch;
        slot.frame = tree
            .add_node()
            .window(
                Rl(vec2(50.0, 100.0)) + Ab(vec2(left, -theme::HOTBAR_BOTTOM_MARGIN)),
                Ab(vec2(theme::SLOT_SIZE, theme::SLOT_SIZE)),
                Anchor::BottomLeft,
            )
            .with_rect(6.0, 2.0, theme::SLOT_BORDER)
            .color_raw::<UiBase>(theme::SLOT_COLOR)
            .entity();
        tree.in_parent(slot.frame, |tree| {
            let inset = (theme::SLOT_SIZE - theme::ICON_SIZE) * 0.5;
            let uv = atlas_upload
                .as_ref()
                .map(|upload| (upload.layer, upload.uv_min, upload.uv_max))
                .unwrap_or((0, vec2(0.0, 0.0), vec2(1.0, 1.0)));
            slot.icon = tree
                .add_node()
                .window(
                    Ab(vec2(inset, 4.0)),
                    Ab(vec2(theme::ICON_SIZE, theme::ICON_SIZE)),
                    Anchor::TopLeft,
                )
                .with_image_uv(uv.0, uv.1, uv.2)
                .color_raw::<UiBase>(vec4(1.0, 1.0, 1.0, 1.0))
                .with_visible(false)
                .entity();
            slot.count = tree
                .add_node()
                .window(
                    Rl(vec2(100.0, 100.0)) + Ab(vec2(-5.0, -1.0)),
                    Ab(vec2(30.0, theme::COUNT_SIZE)),
                    Anchor::BottomRight,
                )
                .with_text("", theme::COUNT_SIZE)
                .text_right()
                .with_text_outline(vec4(0.0, 0.0, 0.0, 1.0), 1.0)
                .color_raw::<UiBase>(theme::TEXT_COLOR)
                .entity();
        });
    }
    hud.hotbar.slots = slots;
}
