use crate::resources::UiHandles;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_ui::components::UiLayoutRoot;
use nightshade_ui::prelude::ui_mark_layout_dirty;
use nightshade_ui::resources::RetainedUiDirty;

pub fn fit_the_interface(
    window: Res<Window>,
    handles: Res<UiHandles>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut roots: Query<&mut UiLayoutRoot>,
) {
    let Some(root) = handles.layout_root else {
        return;
    };
    let Some((width, height)) = window.cached_viewport_size else {
        return;
    };
    let dpi = window.cached_scale_factor.max(0.1);
    let wanted = ((width as f32 / dpi) / theme::UI_EXTENT.x)
        .min((height as f32 / dpi) / theme::UI_EXTENT.y)
        .clamp(theme::MIN_UI_SCALE, theme::MAX_UI_SCALE);
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
