use crate::data::{Ability, Ask, Fighter, Unit};
use crate::queries::{Sites, can_capture, is_stationary, ready_to_deploy};
use crate::resources::{Battle, Campaign, Cursor, Screen};
use crate::theme;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_ui::prelude::*;

pub fn pause_items(from: Screen) -> [bool; 3] {
    [true, from == Screen::Battle, true]
}

pub fn step_index(index: usize, enabled: &[bool], forward: bool) -> usize {
    let count = enabled.len();
    let mut next = index;
    for _ in 0..count {
        next = if forward {
            (next + 1) % count
        } else {
            (next + count - 1) % count
        };
        if enabled[next] {
            return next;
        }
    }
    index
}

pub fn menu_items(battle: &Battle, unit: &Unit, sites: &Sites) -> [bool; 6] {
    let attack_ready = !battle.acted && !(battle.moved && is_stationary(unit, Ability::Attack));
    let skills_ready = !battle.acted
        && unit.class.sheet().abilities.iter().any(|ability| {
            ability.shape().mp <= unit.mp && !(battle.moved && is_stationary(unit, *ability))
        });
    [
        !battle.moved,
        battle.moved && !battle.acted,
        attack_ready,
        skills_ready,
        !battle.acted && can_capture(sites, unit).is_some(),
        true,
    ]
}

pub fn activity_items(campaign: &Campaign, fighter: &Fighter) -> [bool; 6] {
    let ready = ready_to_deploy(fighter);
    let free = campaign.actions > 0 && !fighter.busy && ready;
    [
        free,
        free,
        ready && fighter.marks > 0,
        free && campaign.supplies >= theme::GIFT_COST,
        free,
        ready,
    ]
}

pub fn clicked(frame: &RetainedUiFrame, button: Option<Entity>) -> bool {
    button.is_some_and(|button| ui_button_clicks(frame).any(|clicked| clicked == button))
}

pub fn menu_keys(cursor: &Cursor, actions: &Actions<Ask>) -> (bool, bool, bool) {
    let step = cursor.step;
    (
        actions.just_pressed(Ask::MenuUp) || step.1 > 0,
        actions.just_pressed(Ask::MenuDown) || step.1 < 0,
        actions.just_pressed(Ask::Confirm),
    )
}
