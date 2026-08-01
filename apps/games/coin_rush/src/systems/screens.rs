use crate::resources::{CoinRushResources, Screen};
use nightshade::prelude::*;
use nightshade_ui::prelude::ui_set_visible;
use nightshade_ui::resources::RetainedUiDirty;

pub fn set_screen(
    resources: &mut CoinRushResources,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    screen: Screen,
) {
    resources.round.screen = screen;
    let handles = resources.ui;
    ui_set_visible(dirty, commands, handles.title, screen == Screen::Title);
    ui_set_visible(dirty, commands, handles.pause, screen == Screen::Paused);
    ui_set_visible(dirty, commands, handles.over, screen == Screen::Over);
}
