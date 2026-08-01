use crate::resources::{NightwireResources, Screen};
use nightshade::prelude::*;
use nightshade_ui::prelude::ui_set_visible;
use nightshade_ui::resources::RetainedUiDirty;

pub fn apply(
    mut resources: ResMut<NightwireResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    let screen = resources.night.screen;
    if resources.shown_screen == Some(screen) {
        return;
    }
    resources.shown_screen = Some(screen);
    let handles = resources.ui;
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.title,
        screen == Screen::Title,
    );
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.shift,
        screen == Screen::Shift,
    );
    ui_set_visible(
        &mut dirty,
        &mut commands,
        handles.over,
        screen == Screen::Over,
    );
    resources.rows_dirty = true;
    resources.controls_dirty = true;
}
