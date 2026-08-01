use crate::commands::battle::{attack, end_turn, play_card, use_power};
use crate::data::battle::{Battle, Intent, Pick, Who};
use crate::data::cards::TargetRule;
use crate::queries::battle::{can_play, can_use_power, pick_target, power_rule, targets_for};
use crate::resources::Selection;

pub fn act(
    battle: &mut Battle,
    selection: &mut Selection,
    intent: Intent,
) -> Result<(), &'static str> {
    match intent {
        Intent::Cancel => {
            *selection = Selection::None;
            Ok(())
        }
        Intent::EndTurn => {
            *selection = Selection::None;
            end_turn(battle);
            Ok(())
        }
        Intent::Power => {
            if !can_use_power(battle, Who::Player) {
                return Err("The hero power is spent or too costly");
            }
            if power_rule(battle.player.power) == TargetRule::None {
                *selection = Selection::None;
                use_power(battle, Who::Player, None)
            } else {
                *selection = Selection::Power;
                Ok(())
            }
        }
        Intent::Pick(pick) => act_pick(battle, selection, pick),
    }
}

fn act_pick(
    battle: &mut Battle,
    selection: &mut Selection,
    pick: Pick,
) -> Result<(), &'static str> {
    match (*selection, pick) {
        (Selection::Spell(spell), pick) => {
            if pick == Pick::Hand(spell) {
                *selection = Selection::None;
                return Ok(());
            }
            let Some(target) = pick_target(pick) else {
                *selection = Selection::None;
                return act_pick(battle, selection, pick);
            };
            let rule = battle
                .player
                .hand_card(spell)
                .map(|instance| instance.card.target_rule())
                .unwrap_or(TargetRule::None);
            if !targets_for(battle, Who::Player, rule).contains(&target) {
                return Err("That is not a valid target");
            }
            *selection = Selection::None;
            play_card(battle, Who::Player, spell, Some(target))
        }
        (Selection::Power, pick) => {
            let Some(target) = pick_target(pick) else {
                *selection = Selection::None;
                return act_pick(battle, selection, pick);
            };
            let rule = power_rule(battle.player.power);
            if !targets_for(battle, Who::Player, rule).contains(&target) {
                return Err("That is not a valid target");
            }
            *selection = Selection::None;
            use_power(battle, Who::Player, Some(target))
        }
        (Selection::Attacker(attacker), Pick::Foe(_) | Pick::Hero(Who::Enemy)) => {
            let Some(target) = pick_target(pick) else {
                return Ok(());
            };
            *selection = Selection::None;
            attack(battle, Who::Player, attacker, target)
        }
        (Selection::Attacker(attacker), Pick::Mine(id)) => {
            if attacker == id {
                *selection = Selection::None;
            } else {
                *selection = Selection::None;
                return act_pick(battle, selection, pick);
            }
            Ok(())
        }
        (Selection::Attacker(_), pick) => {
            *selection = Selection::None;
            act_pick(battle, selection, pick)
        }
        (Selection::None, Pick::Hand(id)) => {
            if !can_play(battle, Who::Player, id) {
                return Err("Not enough mana or no room for that card");
            }
            let rule = battle
                .player
                .hand_card(id)
                .map(|instance| instance.card.target_rule())
                .unwrap_or(TargetRule::None);
            if rule == TargetRule::None {
                play_card(battle, Who::Player, id, None)
            } else {
                *selection = Selection::Spell(id);
                Ok(())
            }
        }
        (Selection::None, Pick::Mine(id)) => {
            let can = battle
                .player
                .minion(id)
                .is_some_and(|minion| minion.can_attack());
            if can {
                *selection = Selection::Attacker(id);
                Ok(())
            } else {
                Err("That minion cannot attack yet")
            }
        }
        (Selection::None, Pick::Foe(_) | Pick::Hero(_)) => Ok(()),
    }
}
