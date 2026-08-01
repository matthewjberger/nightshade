use crate::data::TOUCH_CONTROLS;
use crate::resources::{HairpinResources, Lap, Screen};
use nightshade::prelude::Commands;
use nightshade_ui::prelude::ui_set_visible;
use nightshade_ui::resources::RetainedUiDirty;

pub fn set_screen(
    resources: &mut HairpinResources,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
    screen: Screen,
) {
    resources.screen = screen;
    let driving = TOUCH_CONTROLS && screen == Screen::Playing;
    let panels = [
        (resources.ui.title, screen == Screen::Title),
        (resources.ui.pause, screen == Screen::Paused),
        (resources.ui.hud, screen != Screen::Title),
        (resources.ui.pause_pad, driving),
    ];
    for (panel, shown) in panels {
        if let Some(panel) = panel {
            ui_set_visible(dirty, commands, panel, shown);
        }
    }
    for pad in resources.ui.pads.into_iter().flatten() {
        ui_set_visible(dirty, commands, pad, driving);
    }
}

pub fn reset_lap(resources: &mut HairpinResources) {
    resources.lap = Lap::default();
}
