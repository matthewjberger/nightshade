use crate::data::battle::{BOARD_LIMIT, Battle, Instance, POWER_REST_TURNS, Pick, Target, Who};
use crate::data::cards::{Kind, Spell, TargetRule};
use crate::data::decks::HeroPower;
use crate::data::gates::{GateRule, TITHE_SURCHARGE};
use crate::resources::{Cursor, Selection, Zone};

pub fn can_play(battle: &Battle, who: Who, id: u32) -> bool {
    play_block(battle, who, id).is_none()
}

pub fn card_cost(battle: &Battle, who: Who, instance: Instance) -> u32 {
    let mut cost = instance.card.def().cost;
    if instance.upgraded && !instance.card.forge_number() {
        cost = cost.saturating_sub(1);
    }
    if who == Who::Player && battle.rule == GateRule::Tithe && battle.played_this_turn == 0 {
        cost += TITHE_SURCHARGE;
    }
    cost
}

pub fn play_block(battle: &Battle, who: Who, id: u32) -> Option<String> {
    let side = battle.side(who);
    let instance = side.hand_card(id)?;
    let def = instance.card.def();
    let cost = card_cost(battle, who, instance);
    if side.mana < cost {
        let short = cost - side.mana;
        return Some(format!("Needs {short} more mana"));
    }
    let blocked = match def.kind {
        Kind::Minion { .. } => {
            if side.board.len() < BOARD_LIMIT {
                None
            } else {
                Some("No room on your line")
            }
        }
        Kind::Spell { effect, target } => match effect {
            Spell::Summon(..) => {
                if side.board.len() < BOARD_LIMIT {
                    None
                } else {
                    Some("No room on your line")
                }
            }
            Spell::Buff(..) | Spell::Shield => {
                if side.board.is_empty() {
                    Some("Needs a minion of yours")
                } else {
                    None
                }
            }
            _ => {
                if target == TargetRule::None || !targets_for(battle, who, target).is_empty() {
                    None
                } else {
                    Some("Nothing to aim at")
                }
            }
        },
    };
    blocked.map(str::to_string)
}

pub fn pending_damage(battle: &Battle, selection: Selection) -> i32 {
    match selection {
        Selection::None => 0,
        Selection::Attacker(id) => battle
            .player
            .minion(id)
            .map(|minion| minion.attack)
            .unwrap_or(0),
        Selection::Spell(id) => {
            let Some(instance) = battle.player.hand_card(id) else {
                return 0;
            };
            let Kind::Spell { effect, .. } = instance.card.def().kind else {
                return 0;
            };
            let Spell::Damage(amount) = effect else {
                return 0;
            };
            let element = instance.card.def().element;
            let chained = element != crate::data::cards::Element::Neutral
                && battle.player.turn_element == Some(element);
            let forged = instance.upgraded && instance.card.forge_number();
            amount + battle.player.spell_bonus + i32::from(chained) + i32::from(forged)
        }
        Selection::Power => match battle.player.power {
            HeroPower::Spark => 1,
            _ => 0,
        },
    }
}

pub fn targets_for(battle: &Battle, who: Who, rule: TargetRule) -> Vec<Target> {
    let foe = crate::data::battle::other(who);
    let mut targets = Vec::new();
    match rule {
        TargetRule::None => {}
        TargetRule::AnyEnemy => {
            targets.push(Target::Hero(foe));
            targets.extend(
                battle
                    .side(foe)
                    .board
                    .iter()
                    .map(|minion| Target::Minion(foe, minion.id)),
            );
        }
        TargetRule::EnemyMinion => {
            targets.extend(
                battle
                    .side(foe)
                    .board
                    .iter()
                    .map(|minion| Target::Minion(foe, minion.id)),
            );
        }
        TargetRule::AnyCharacter => {
            targets.push(Target::Hero(who));
            targets.push(Target::Hero(foe));
            for side in [who, foe] {
                targets.extend(
                    battle
                        .side(side)
                        .board
                        .iter()
                        .map(|minion| Target::Minion(side, minion.id)),
                );
            }
        }
        TargetRule::FriendlyMinion => {
            targets.extend(
                battle
                    .side(who)
                    .board
                    .iter()
                    .map(|minion| Target::Minion(who, minion.id)),
            );
        }
    }
    targets
}

pub fn attack_targets(battle: &Battle, who: Who) -> Vec<Target> {
    let foe = crate::data::battle::other(who);
    let side = battle.side(foe);
    if side.has_taunt() {
        side.board
            .iter()
            .filter(|minion| minion.guard)
            .map(|minion| Target::Minion(foe, minion.id))
            .collect()
    } else {
        let mut targets = vec![Target::Hero(foe)];
        targets.extend(
            side.board
                .iter()
                .map(|minion| Target::Minion(foe, minion.id)),
        );
        targets
    }
}

pub fn power_rule(power: HeroPower) -> TargetRule {
    match power {
        HeroPower::Spark => TargetRule::AnyEnemy,
        HeroPower::Mend => TargetRule::AnyCharacter,
        HeroPower::Sprout | HeroPower::Bolt => TargetRule::None,
    }
}

pub fn can_use_power(battle: &Battle, who: Who) -> bool {
    let side = battle.side(who);
    if side.power_used || side.mana < side.power_cost || battle.active != who {
        return false;
    }
    match side.power {
        HeroPower::Sprout => side.board.len() < BOARD_LIMIT,
        HeroPower::Mend => power_rested(battle, who),
        _ => true,
    }
}

pub fn power_rested(battle: &Battle, who: Who) -> bool {
    let side = battle.side(who);
    side.power_turn == 0 || battle.turn >= side.power_turn + POWER_REST_TURNS
}

pub fn pick_at_cursor(battle: &Battle, cursor: Cursor) -> Option<Pick> {
    match cursor.zone {
        Zone::Hand => battle
            .player
            .hand
            .get(cursor.index)
            .map(|instance| Pick::Hand(instance.id)),
        Zone::Board => battle
            .player
            .board
            .get(cursor.index)
            .map(|minion| Pick::Mine(minion.id)),
        Zone::Enemy => match battle.enemy.board.get(cursor.index) {
            Some(minion) => Some(Pick::Foe(minion.id)),
            None => Some(Pick::Hero(Who::Enemy)),
        },
    }
}

pub fn zone_len(battle: &Battle, zone: Zone) -> usize {
    match zone {
        Zone::Hand => battle.player.hand.len(),
        Zone::Board => battle.player.board.len(),
        Zone::Enemy => battle.enemy.board.len() + 1,
    }
}

pub fn pick_from_id(battle: &Battle, id: u32) -> Option<Pick> {
    if battle.player.hand_card(id).is_some() {
        return Some(Pick::Hand(id));
    }
    if battle.player.minion(id).is_some() {
        return Some(Pick::Mine(id));
    }
    if battle.enemy.minion(id).is_some() {
        return Some(Pick::Foe(id));
    }
    None
}

pub fn pick_target(pick: Pick) -> Option<Target> {
    match pick {
        Pick::Hand(_) => None,
        Pick::Mine(id) => Some(Target::Minion(Who::Player, id)),
        Pick::Foe(id) => Some(Target::Minion(Who::Enemy, id)),
        Pick::Hero(who) => Some(Target::Hero(who)),
    }
}
