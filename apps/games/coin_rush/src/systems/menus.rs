use crate::data::Ask;
use crate::resources::{CoinRushResources, Screen};
use crate::systems::round::{Coins, Players, start_round};
use crate::systems::screens::set_screen;
use nightshade::prelude::*;
use nightshade_countdown::queries::gone;
use nightshade_countdown::{Remaining, Running};
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_ui::components::UiButtonData;
use nightshade_ui::prelude::ui_clicked;
use nightshade_ui::resources::RetainedUiDirty;

pub fn update(
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<CoinRushResources>,
    mut window: ResMut<Window>,
    mut remaining: ResMut<Remaining>,
    mut running: ResMut<Running>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut buttons: Query<(&UiButtonData,)>,
    mut coins: Coins<'_, '_>,
    mut players: Players<'_, '_>,
) {
    let resources = &mut *resources;
    let handles = resources.ui;
    let pause_pressed = actions.just_pressed(Ask::Pause);

    match resources.round.screen {
        Screen::Title => {
            if ui_clicked(&mut buttons, handles.play_button) {
                start_round(
                    resources,
                    &mut remaining,
                    &mut running,
                    &mut dirty,
                    &mut commands,
                    &mut coins,
                    &mut players,
                );
            } else if ui_clicked(&mut buttons, handles.quit_button) {
                window.should_exit = true;
            }
        }
        Screen::Playing => {
            if pause_pressed {
                set_screen(resources, &mut dirty, &mut commands, Screen::Paused);
            }
        }
        Screen::Paused => {
            if ui_clicked(&mut buttons, handles.resume_button) || pause_pressed {
                set_screen(resources, &mut dirty, &mut commands, Screen::Playing);
            } else if ui_clicked(&mut buttons, handles.restart_button) {
                start_round(
                    resources,
                    &mut remaining,
                    &mut running,
                    &mut dirty,
                    &mut commands,
                    &mut coins,
                    &mut players,
                );
            } else if ui_clicked(&mut buttons, handles.pause_title_button) {
                set_screen(resources, &mut dirty, &mut commands, Screen::Title);
            }
        }
        Screen::Over => {
            if ui_clicked(&mut buttons, handles.again_button) {
                start_round(
                    resources,
                    &mut remaining,
                    &mut running,
                    &mut dirty,
                    &mut commands,
                    &mut coins,
                    &mut players,
                );
            } else if ui_clicked(&mut buttons, handles.over_title_button) {
                set_screen(resources, &mut dirty, &mut commands, Screen::Title);
            }
        }
    }

    running.0 = resources.round.screen == Screen::Playing && !gone(&remaining);
}
