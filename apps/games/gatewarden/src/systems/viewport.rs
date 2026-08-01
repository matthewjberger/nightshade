use crate::contract::Screen;
use crate::resources::GatewardenResources;
use crate::theme::{
    BATTLE_EXTENT, MAP_EXTENT, MAX_UI_SCALE, MIN_UI_SCALE, OVER_EXTENT, SCENE_EXTENT, SHOP_EXTENT,
    TITLE_EXTENT,
};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::components::UiLayoutRoot;
use nightshade_ui::prelude::ui_mark_layout_dirty;
use nightshade_ui::resources::RetainedUiDirty;

pub fn scale(
    window: Res<Window>,
    resources: Res<GatewardenResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut roots: Query<&mut UiLayoutRoot>,
) {
    let Some(root) = resources.ui.layout_root else {
        return;
    };
    let Some((width, height)) = window.cached_viewport_size else {
        return;
    };
    let dpi = window.cached_scale_factor.max(0.1);
    let logical_width = width as f32 / dpi;
    let logical_height = height as f32 / dpi;
    let extent = match resources.screen {
        Screen::Title => TITLE_EXTENT,
        Screen::Map => MAP_EXTENT,
        Screen::Scene => SCENE_EXTENT,
        Screen::Battle => BATTLE_EXTENT,
        Screen::Shop => SHOP_EXTENT,
        Screen::Over => OVER_EXTENT,
    };
    let wanted = (logical_width / extent.x)
        .min(logical_height / extent.y)
        .clamp(MIN_UI_SCALE, MAX_UI_SCALE);
    let mut changed = false;
    roots.for_entity(root, |layout_root| {
        if (layout_root.absolute_scale - wanted).abs() > 0.005 {
            layout_root.absolute_scale = wanted;
            changed = true;
        }
    });
    if changed {
        ui_mark_layout_dirty(&mut dirty);
    }
}
