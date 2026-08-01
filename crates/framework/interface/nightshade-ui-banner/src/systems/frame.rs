use crate::queries::built;
use crate::resources::{Banner, BannerHandles, BannerShown};
use nightshade::prelude::*;
use nightshade_ui::data::state::UiBase;
use nightshade_ui::prelude::{ui_set_color, ui_set_text, ui_set_visible};
use nightshade_ui::resources::RetainedUiDirty;

pub fn show_the_banner(
    banner: Res<Banner>,
    handles: Res<BannerHandles>,
    mut shown: ResMut<BannerShown>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    if !built(&handles) {
        return;
    }
    if shown.line != banner.line {
        shown.line = banner.line.clone();
        ui_set_text(&mut commands, handles.text, &banner.line);
    }
    if shown.color != Some(banner.color) {
        shown.color = Some(banner.color);
        ui_set_color::<UiBase>(&mut commands, handles.text, banner.color);
    }
    if shown.shown != Some(banner.shown) {
        shown.shown = Some(banner.shown);
        ui_set_visible(&mut dirty, &mut commands, handles.panel, banner.shown);
    }
}
