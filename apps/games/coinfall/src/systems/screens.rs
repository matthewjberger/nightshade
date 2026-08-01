use crate::resources::{CoinfallResources, Screen};
use nightshade::prelude::*;
use nightshade_scene::prelude::Visibility;
use nightshade_ui::prelude::ui_set_visible;
use nightshade_ui::resources::RetainedUiDirty;
use nightshade_ui_banner::Banner;

pub fn apply(
    mut resources: ResMut<CoinfallResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut banner: ResMut<Banner>,
    mut commands: Commands,
) {
    let Some(screen) = resources.run.wanted_screen.take() else {
        return;
    };
    resources.run.screen = screen;

    let handles = resources.hud;
    let panels = [
        (handles.title, screen == Screen::Title),
        (handles.overworld, screen == Screen::Overworld),
        (handles.pause, screen == Screen::Paused),
        (handles.cleared, screen == Screen::Cleared),
    ];
    banner.shown = screen == Screen::Playing || screen == Screen::Paused;
    for (node, visible) in panels {
        if node != Entity::default() {
            ui_set_visible(&mut dirty, &mut commands, node, visible);
        }
    }

    if let Some(player) = resources.player {
        let visible = screen == Screen::Playing || screen == Screen::Paused;
        commands.set(player, Visibility { visible });
    }
}
