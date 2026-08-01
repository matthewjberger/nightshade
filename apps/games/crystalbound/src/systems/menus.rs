use crate::commands::{assign_job, buy, revive, show_toast, travel, unlock_job};
use crate::contract::{BattleCommand, Job, Place, Ware};
use crate::data::Ask;
use crate::queries::heroes_standing;
use crate::resources::{
    Battle, Field, Hud, Party, Phase, Screen, ScreenRequest, ShownScreen, Toast,
};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;

pub fn read_the_state(mut shown_screen: ResMut<ShownScreen>, state_now: Res<State<Screen>>) {
    let now = state_now.current;
    shown_screen.0 = now;
}

pub fn apply_the_request(
    mut screen_request: ResMut<ScreenRequest>,
    mut state_next: ResMut<NextState<Screen>>,
) {
    let wanted = screen_request.wanted.take();
    if let Some(wanted) = wanted {
        state_next.pending = Some(wanted);
    }
}

pub fn watch_the_party(
    screen: Res<ShownScreen>,
    party: Res<Party>,
    mut request: ResMut<ScreenRequest>,
) {
    if screen.0 == Screen::Field && heroes_standing(&party) == 0 {
        request.wanted = Some(Screen::Defeat);
    }
}

pub fn drive_the_menus(
    actions: Res<Actions<Ask>>,
    frame: Res<RetainedUiFrame>,
    screen: Res<ShownScreen>,
    hud: Res<Hud>,
    mut party: ResMut<Party>,
    mut battle: ResMut<Battle>,
    mut field: ResMut<Field>,
    mut toast: ResMut<Toast>,
    mut request: ResMut<ScreenRequest>,
    mut window: ResMut<Window>,
) {
    let clicks: Vec<Entity> = ui_button_clicks(&frame).collect();
    let handles = hud.handles;
    let escape = actions.just_pressed(Ask::Pause);
    let accept = actions.just_pressed(Ask::Accept);
    let hit = |node: Option<Entity>| node.is_some_and(|node| clicks.contains(&node));

    match screen.0 {
        Screen::Title => {
            if hit(handles.title_play) || accept {
                request.wanted = Some(Screen::Field);
            } else if hit(handles.title_quit) {
                window.should_exit = true;
            }
        }
        Screen::Field => {
            if escape {
                request.wanted = Some(Screen::Paused);
            }
        }
        Screen::Paused => {
            if hit(handles.paused_resume) || escape {
                request.wanted = Some(Screen::Field);
            } else if hit(handles.paused_title) {
                request.wanted = Some(Screen::Title);
            }
        }
        Screen::Shop => {
            if hit(handles.shop_leave) || escape {
                request.wanted = Some(Screen::Field);
                return;
            }
            for (slot, ware) in Ware::ALL.iter().enumerate() {
                if !hit(handles.shop_buttons[slot]) {
                    continue;
                }
                match buy(&mut party, *ware) {
                    Ok(note) | Err(note) => show_toast(&mut toast, note),
                }
            }
        }
        Screen::Jobs => {
            if hit(handles.jobs_leave) || escape {
                request.wanted = Some(Screen::Field);
                return;
            }
            for slot in 0..4 {
                if hit(handles.jobs_heroes[slot]) {
                    party.chosen = slot;
                }
            }
            if hit(handles.jobs_unlock) {
                match unlock_job(&mut party) {
                    Ok(note) | Err(note) => show_toast(&mut toast, note),
                }
            }
            for (slot, job) in Job::LADDER.iter().enumerate() {
                if !hit(handles.jobs_list[slot]) {
                    continue;
                }
                if !party.unlocked.contains(job) {
                    show_toast(&mut toast, "The crystal has not opened that way.");
                    continue;
                }
                let chosen = party.chosen;
                let note = assign_job(&mut party, chosen, *job);
                show_toast(&mut toast, note);
            }
        }
        Screen::Battle => {
            if battle.phase != Phase::Choosing {
                return;
            }
            for (slot, command) in BattleCommand::ALL.iter().enumerate() {
                if hit(handles.command_buttons[slot]) || actions.just_pressed(Ask::COMMANDS[slot]) {
                    battle.command = Some(*command);
                }
            }
        }
        Screen::Defeat => {
            if hit(handles.defeat_title) || accept {
                revive(&mut party);
                party.gold /= 2;
                travel(&mut field, Place::Overworld, None);
                show_toast(&mut toast, "You wake on the moor, lighter of purse.");
                request.wanted = Some(Screen::Field);
            }
        }
    }
}
