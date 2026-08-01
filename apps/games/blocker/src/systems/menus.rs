use crate::data::TOGGLES;
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_hud::resources::ChatPanel;
use blocker_player::components::Motion;
use blocker_player::data::ask::Ask;
use blocker_pocket::resources::OpenChest;
use blocker_screen::data::screen::Screen;
use blocker_screens::commands::leave_the_chest;
use blocker_screens::resources::Screens;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::RetainedUiInteraction;

pub fn handle_the_keys(
    state: Res<State<Screen>>,
    actions: Res<Actions<Ask>>,
    interaction: Res<RetainedUiInteraction>,
    chat_panel: Res<ChatPanel>,
    hero: Res<Hero>,
    mut open: ResMut<OpenChest>,
    mut screens: ResMut<Screens>,
    mut next: ResMut<NextState<Screen>>,
    mut bodies: Query<(&Motion,)>,
) {
    if interaction.focused_entity == Some(chat_panel.entry) {
        return;
    }
    let mode = read_the_hero(&hero, &mut bodies, |(motion,)| *motion).unwrap_or(Motion::Walk);
    let escape = actions.just_pressed(Ask::Back);
    let pocket = actions.just_pressed(Ask::Pocket);
    let current = state.current;
    if let Some(toggle) = TOGGLES.iter().find(|toggle| toggle.screen == current) {
        if escape || actions.just_pressed(toggle.ask) {
            next.pending = Some(Screen::Playing);
        }
        return;
    }
    match current {
        Screen::Title | Screen::Loading | Screen::Setup => {}
        Screen::NewWorld => {
            if escape {
                next.pending = Some(Screen::Title);
            }
        }
        Screen::Playing => {
            let opened = TOGGLES.iter().find(|toggle| {
                actions.just_pressed(toggle.ask)
                    && !(toggle.screen == Screen::Inventory && mode == Motion::Watch)
            });
            if let Some(toggle) = opened {
                screens.return_to = Screen::Playing;
                next.pending = Some(toggle.screen);
            } else if escape {
                screens.return_to = Screen::Playing;
                next.pending = Some(Screen::Settings);
            }
        }
        Screen::Settings => {
            if escape || pocket {
                next.pending = Some(screens.return_to);
            }
        }
        Screen::Chest => {
            if escape || pocket {
                leave_the_chest(&mut open, &mut next);
            }
        }
        Screen::Crafting | Screen::Dialogue => {
            if escape {
                next.pending = Some(Screen::Playing);
            }
        }
        Screen::Shop => {
            if escape || pocket {
                next.pending = Some(Screen::Playing);
            }
        }
        Screen::Inventory | Screen::Map | Screen::Character | Screen::Journal => {}
    }
}
