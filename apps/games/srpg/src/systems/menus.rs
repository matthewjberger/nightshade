use crate::commands::campaign::{heal_the_camp, take_the_next_chapter};
use crate::commands::edit_structure;
use crate::commands::menus::{aim, hold_the_game, start_the_run, take_back_the_move};
use crate::commands::save::{forget_the_run, read_the_run, restore_the_run};
use crate::commands::{capture, clear_highlights, log, reset_battle, set_phase, show_moves, story};
use crate::data::{Ability, Ask, Team};
use crate::queries::Structures;
use crate::queries::menu::{clicked, menu_items, menu_keys, pause_items, step_index};
use crate::queries::{
    can_capture, destinations, is_stationary, menu_step, reachable, roster, roster_index,
};
use crate::resources::{
    Battle, CampPhase, Campaign, Cursor, Highlights, Pause, Phase, Screen, Terrain, UiHandles,
};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui::prelude::*;

pub fn handle_the_keys(
    screen_state: Res<State<Screen>>,
    battle: Res<Battle>,
    actions: Res<Actions<Ask>>,
    mut campaign: ResMut<Campaign>,
    mut pause: ResMut<Pause>,
    mut window: ResMut<Window>,
    mut next: ResMut<NextState<Screen>>,
    mut fighters: crate::queries::Fighters,
) {
    let screen = screen_state.current;
    let phase = battle.phase;
    let escape = actions.just_pressed(Ask::Pause);
    let start = actions.just_pressed(Ask::Begin);
    let quit = actions.just_pressed(Ask::Quit);
    let called = actions.just_pressed(Ask::March);
    let sub_phase = matches!(
        phase,
        Phase::ChooseMove | Phase::ChooseAbility | Phase::ChooseTarget
    );
    match screen {
        Screen::Title => {
            if start {
                start_the_run(&mut campaign, &mut next, &mut fighters);
            }
        }
        Screen::Camp => {
            if escape && campaign.phase == CampPhase::Roam {
                hold_the_game(&mut pause, &mut next, Screen::Camp);
                return;
            }
            let ready = campaign.phase == CampPhase::Roam;
            if ready && called {
                campaign.march = true;
                story(&mut campaign, "The squad forms up and marches out.");
            }
        }
        Screen::Battle => {
            if escape && !sub_phase {
                hold_the_game(&mut pause, &mut next, Screen::Battle);
            }
        }
        Screen::Paused => {
            if escape {
                next.pending = Some(pause.from);
            }
        }
        Screen::Prologue | Screen::Over => {}
    }
    if quit {
        window.should_exit = true;
    }
}

pub fn title_buttons(
    handles: Res<UiHandles>,
    frame: Res<RetainedUiFrame>,
    mut campaign: ResMut<Campaign>,
    mut window: ResMut<Window>,
    mut next: ResMut<NextState<Screen>>,
    mut fighters: crate::queries::Fighters,
) {
    if clicked(&frame, handles.start_button) {
        forget_the_run();
        start_the_run(&mut campaign, &mut next, &mut fighters);
    } else if clicked(&frame, handles.continue_button)
        && let Some(save) = read_the_run()
    {
        restore_the_run(&mut campaign, &mut fighters, &save);
        next.pending = Some(Screen::Camp);
    } else if clicked(&frame, handles.title_quit_button) {
        window.should_exit = true;
    }
}

pub fn camp_buttons(
    handles: Res<UiHandles>,
    frame: Res<RetainedUiFrame>,
    mut campaign: ResMut<Campaign>,
) {
    if clicked(&frame, handles.march_button) {
        campaign.march = true;
        story(&mut campaign, "The squad forms up and marches out.");
    }
}

pub fn pause_buttons(
    handles: Res<UiHandles>,
    frame: Res<RetainedUiFrame>,
    actions: Res<Actions<Ask>>,
    mut pause: ResMut<Pause>,
    mut battle: ResMut<Battle>,
    mut highlights: ResMut<Highlights>,
    mut campaign: ResMut<Campaign>,
    mut next: ResMut<NextState<Screen>>,
) {
    let step = menu_step(&actions);
    let up = step > 0;
    let down = step < 0;
    let confirm = actions.just_pressed(Ask::Confirm);
    let from = pause.from;
    let enabled = pause_items(from);
    let mut picked = None;
    for (item, button) in handles.pause_buttons.iter().enumerate() {
        if clicked(&frame, *button) && enabled.get(item).copied().unwrap_or(false) {
            picked = Some(item);
        }
    }
    if up {
        pause.index = step_index(pause.index, &enabled, false);
    }
    if down {
        pause.index = step_index(pause.index, &enabled, true);
    }
    if confirm && picked.is_none() && enabled[pause.index] {
        picked = Some(pause.index);
    }
    let Some(item) = picked else {
        return;
    };
    if item == 0 {
        next.pending = Some(from);
        return;
    }
    reset_battle(&mut battle);
    clear_highlights(&mut highlights);
    if item == 1 {
        campaign.arrival = true;
        campaign.settled = false;
        story(
            &mut campaign,
            "The squad breaks off and falls back to camp.",
        );
        next.pending = Some(Screen::Camp);
        return;
    }
    next.pending = Some(Screen::Title);
}

pub fn over_buttons(
    handles: Res<UiHandles>,
    frame: Res<RetainedUiFrame>,
    actions: Res<Actions<Ask>>,
    mut battle: ResMut<Battle>,
    mut highlights: ResMut<Highlights>,
    mut campaign: ResMut<Campaign>,
    mut next: ResMut<NextState<Screen>>,
    mut fighters: crate::queries::Fighters,
) {
    let onward = clicked(&frame, handles.retry_button) || actions.just_pressed(Ask::Onward);
    if onward {
        let won = battle.outcome == Some(Team::Player);
        if !won {
            heal_the_camp(&mut fighters);
        }
        reset_battle(&mut battle);
        clear_highlights(&mut highlights);
        let more = take_the_next_chapter(&mut campaign, won);
        next.pending = Some(if more { Screen::Camp } else { Screen::Title });
    } else if clicked(&frame, handles.over_quit_button) {
        reset_battle(&mut battle);
        clear_highlights(&mut highlights);
        next.pending = Some(Screen::Title);
    }
}

pub fn command_menu(
    handles: Res<UiHandles>,
    frame: Res<RetainedUiFrame>,
    actions: Res<Actions<Ask>>,
    terrain: Res<Terrain>,
    mut cursor: ResMut<Cursor>,
    mut battle: ResMut<Battle>,
    mut highlights: ResMut<Highlights>,
    mut rig: ResMut<crate::resources::CameraRig>,
    mut units: crate::queries::Units,
    mut transforms: Query<(&mut LocalTransform,)>,
    mut structures: Structures<'_, '_>,
) {
    if battle.phase != Phase::Menu {
        return;
    }
    let Some(active) = battle.active else {
        return;
    };
    let roster = roster(&mut units);
    let Some(index) = roster_index(&roster, active) else {
        set_phase(&mut battle, Phase::TurnEnd);
        return;
    };
    let unit = roster[index].1.clone();
    let site_rows = crate::queries::sites(&mut structures);
    let (up, down, confirm) = menu_keys(&cursor, &actions);
    let enabled = menu_items(&battle, &unit, &site_rows);

    let mut picked = None;
    for (item, button) in handles.menu_buttons.iter().enumerate() {
        if clicked(&frame, Some(*button)) && enabled.get(item).copied().unwrap_or(false) {
            picked = Some(item);
        }
    }
    {
        if battle.menu_index >= enabled.len() || !enabled[battle.menu_index] {
            battle.menu_index =
                step_index(battle.menu_index.min(enabled.len() - 1), &enabled, true);
        }
        if up {
            battle.menu_index = step_index(battle.menu_index, &enabled, false);
        }
        if down {
            battle.menu_index = step_index(battle.menu_index, &enabled, true);
        }
        if confirm && picked.is_none() && enabled[battle.menu_index] {
            picked = Some(battle.menu_index);
        }
    }

    match picked {
        Some(0) => {
            let reach = reachable(&terrain, &roster, &unit);
            let tiles = destinations(&terrain, &reach, &roster, &unit);
            show_moves(&mut highlights, tiles);
            battle.reachable = reach;
            set_phase(&mut battle, Phase::ChooseMove);
        }
        Some(1) => {
            take_back_the_move(
                &mut battle,
                &terrain,
                &mut cursor,
                &mut rig,
                &mut units,
                &mut transforms,
                active,
            );
        }
        Some(2) => aim(
            &terrain,
            &mut highlights,
            &mut cursor,
            &mut rig,
            &mut battle,
            &roster,
            &unit,
            Ability::Attack,
        ),
        Some(3) => {
            battle.ability_index = 0;
            set_phase(&mut battle, Phase::ChooseAbility);
        }
        Some(4) => {
            if let Some(site_index) = can_capture(&site_rows, &unit) {
                let entity = site_rows[site_index].0;
                let kind = site_rows[site_index].1.kind;
                edit_structure(&mut structures, entity, |structure| {
                    capture(structure, unit.team)
                });
                log(
                    &mut battle,
                    format!("{} captures the {}.", unit.name, kind.name()),
                );
                battle.acted = true;
                set_phase(&mut battle, Phase::TurnEnd);
            }
        }
        Some(5) => {
            log(&mut battle, format!("{} holds position.", unit.name));
            set_phase(&mut battle, Phase::TurnEnd);
        }
        _ => {}
    }
}

pub fn ability_menu(
    handles: Res<UiHandles>,
    frame: Res<RetainedUiFrame>,
    actions: Res<Actions<Ask>>,
    terrain: Res<Terrain>,
    mut cursor: ResMut<Cursor>,
    mut battle: ResMut<Battle>,
    mut highlights: ResMut<Highlights>,
    mut rig: ResMut<crate::resources::CameraRig>,
    mut units: crate::queries::Units,
) {
    if battle.phase != Phase::ChooseAbility {
        return;
    }
    let Some(active) = battle.active else {
        return;
    };
    let roster = roster(&mut units);
    let Some(index) = roster_index(&roster, active) else {
        return;
    };
    let unit = roster[index].1.clone();
    let abilities = unit.class.sheet().abilities;
    let moved = battle.moved;
    let enabled: Vec<bool> = abilities
        .iter()
        .map(|ability| ability.shape().mp <= unit.mp && !(moved && is_stationary(&unit, *ability)))
        .collect();
    let (up, down, confirm) = menu_keys(&cursor, &actions);
    let cancel = cursor.cancel;

    if cancel {
        set_phase(&mut battle, Phase::Menu);
        return;
    }

    let mut picked = None;
    for (item, button) in handles.ability_buttons.iter().enumerate() {
        if clicked(&frame, Some(*button)) && enabled.get(item).copied().unwrap_or(false) {
            picked = Some(item);
        }
    }
    {
        if battle.ability_index >= abilities.len() {
            battle.ability_index = 0;
        }
        if up {
            battle.ability_index = step_index(battle.ability_index, &enabled, false);
        }
        if down {
            battle.ability_index = step_index(battle.ability_index, &enabled, true);
        }
        if confirm && picked.is_none() && enabled.get(battle.ability_index) == Some(&true) {
            picked = Some(battle.ability_index);
        }
    }
    if let Some(item) = picked
        && let Some(ability) = abilities.get(item)
    {
        aim(
            &terrain,
            &mut highlights,
            &mut cursor,
            &mut rig,
            &mut battle,
            &roster,
            &unit,
            *ability,
        );
    }
}
