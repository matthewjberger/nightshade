use crate::resources::Handles;
use nightshade::prelude::*;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

pub fn show_the_card(
    handles: Res<Handles>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    ui_set_visible(&mut dirty, &mut commands, handles.card, true);
    ui_set_visible(&mut dirty, &mut commands, handles.play_root, false);
}

pub fn show_the_scene(
    handles: Res<Handles>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    ui_set_visible(&mut dirty, &mut commands, handles.card, false);
    ui_set_visible(&mut dirty, &mut commands, handles.play_root, true);
}
