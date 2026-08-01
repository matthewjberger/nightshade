use crate::commands::actions::act;
use crate::commands::battle::{attack, end_turn, play_card, use_power};
use crate::commands::cursor::move_cursor;
use crate::commands::run::{self, BattleEnd};
use crate::commands::screens::set_screen;
use crate::commands::tutorial::record;
use crate::components::Instance;
use crate::contract::Screen;
use crate::data::ask::{Ask, HAND_CARDS};
use crate::data::battle::{Intent, Pick, Who};
use crate::data::tutorial::REFUSALS;
use crate::queries::ai::{Action, next_action};
use crate::queries::battle::{pick_at_cursor, pick_from_id};
use crate::queries::tutorial::{deed, gate, permits};
use crate::resources::{GatewardenResources, Selection};
use crate::theme::{AI_THINK_SECONDS, BANNER_SECONDS, OUTCOME_HOLD_SECONDS};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_ui::components::UiButtonData;
use nightshade_ui::prelude::ui_clicked;
use nightshade_ui::resources::RetainedUiDirty;

pub fn update(
    input: Res<Input>,
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<GatewardenResources>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut buttons: Query<(&UiButtonData,)>,
    mut cards: Query<(&Instance,)>,
) {
    let resources = &mut *resources;
    if resources.screen != Screen::Battle || resources.paused {
        return;
    }
    let Some(battle) = resources.battle.as_mut() else {
        return;
    };
    let theater = &mut resources.theater;
    let busy = theater.delay > 0.0 || !battle.events.is_empty() || theater.lunge.is_some();

    if battle.outcome.is_some() {
        if busy {
            return;
        }
        if !theater.ending {
            theater.ending = true;
            theater.delay = OUTCOME_HOLD_SECONDS;
            resources.selection = Selection::None;
            return;
        }
        let screen = match run::finish_battle(resources) {
            BattleEnd::Won => Screen::Scene,
            BattleEnd::Lost => Screen::Over,
        };
        set_screen(resources, &mut dirty, &mut commands, screen);
        return;
    }

    if battle.active == Who::Enemy {
        if busy {
            return;
        }
        theater.think -= time.delta_time;
        if theater.think > 0.0 {
            return;
        }
        theater.think = AI_THINK_SECONDS;
        match next_action(battle) {
            Action::Play { id, target } => {
                if play_card(battle, Who::Enemy, id, target).is_err() {
                    end_turn(battle);
                }
            }
            Action::Attack { attacker, target } => {
                if attack(battle, Who::Enemy, attacker, target).is_err() {
                    end_turn(battle);
                }
            }
            Action::Power { target } => {
                if use_power(battle, Who::Enemy, target).is_err() {
                    end_turn(battle);
                }
            }
            Action::End => end_turn(battle),
        }
        return;
    }

    move_cursor(&input, &actions, battle, &mut resources.cursor);

    let handles = resources.ui;
    let mut intent = None;
    let mut card_nodes = Vec::new();
    cards.for_each_mut(|entity, (id,)| card_nodes.push((entity, id.0)));
    for (entity, id) in card_nodes {
        if ui_clicked(&mut buttons, entity) {
            intent = pick_from_id(battle, id).map(Intent::Pick);
        }
    }
    if handles
        .enemy_hero
        .is_some_and(|entity| ui_clicked(&mut buttons, entity))
    {
        intent = Some(Intent::Pick(Pick::Hero(Who::Enemy)));
    }
    if handles
        .player_hero
        .is_some_and(|entity| ui_clicked(&mut buttons, entity))
    {
        intent = Some(Intent::Pick(Pick::Hero(Who::Player)));
    }
    if handles
        .end_turn_button
        .is_some_and(|entity| ui_clicked(&mut buttons, entity))
        || actions.just_pressed(Ask::EndTurn)
    {
        intent = Some(Intent::EndTurn);
    }
    if handles
        .power_button
        .is_some_and(|entity| ui_clicked(&mut buttons, entity))
        || actions.just_pressed(Ask::Power)
    {
        intent = Some(Intent::Power);
    }
    if actions.just_pressed(Ask::Cancel) {
        intent = Some(Intent::Cancel);
    }
    for (index, card) in HAND_CARDS.iter().enumerate() {
        if actions.just_pressed(*card)
            && let Some(instance) = battle.player.hand.get(index)
        {
            intent = Some(Intent::Pick(Pick::Hand(instance.id)));
        }
    }
    if actions.just_pressed(Ask::Pick)
        && !resources.tutorial.swallow_pick
        && let Some(pick) = pick_at_cursor(battle, resources.cursor)
    {
        intent = Some(Intent::Pick(pick));
    }

    let Some(intent) = intent else {
        return;
    };
    if busy {
        return;
    }
    if let Some(current) = gate(resources.tutorial)
        && !permits(current, battle, resources.selection, &intent)
    {
        let refusal = REFUSALS[resources.tutorial.refusals % REFUSALS.len()];
        resources.tutorial.refusals += 1;
        theater.banner = refusal.to_string();
        theater.banner_timer = BANNER_SECONDS;
        return;
    }
    let done = deed(battle, resources.selection, &intent);
    let outcome = act(battle, &mut resources.selection, intent);
    match outcome {
        Ok(()) => {
            if let Some(done) = done {
                record(&mut resources.tutorial, done);
            }
        }
        Err(message) => {
            theater.banner = message.to_string();
            theater.banner_timer = BANNER_SECONDS;
        }
    }
    if battle.active == Who::Enemy {
        theater.think = AI_THINK_SECONDS;
        resources.selection = Selection::None;
    }
}
