use crate::data::battle::{Battle, Intent, Pick, Target, Who};
use crate::data::cards::{CardId, TargetRule};
use crate::data::tutorial::{Aim, Deed, Focus, Gate, LESSONS};
use crate::queries::battle::pick_target;
use crate::resources::{Selection, Tutorial};

pub fn aim_of(battle: &Battle, target: Target) -> Option<Aim> {
    match target {
        Target::Hero(Who::Enemy) => Some(Aim::Warden),
        Target::Minion(Who::Enemy, id) => battle
            .enemy
            .minion(id)
            .map(|minion| Aim::Minion(minion.card)),
        _ => None,
    }
}

pub fn gate_opens(gate: Gate, deed: Deed) -> bool {
    match (gate, deed) {
        (Gate::Play(card), Deed::Played(done)) => card == done,
        (Gate::Attack(card, aim), Deed::Attacked(done, hit)) => card == done && aim == hit,
        (Gate::Cast(card, aim), Deed::Cast(done, hit)) => card == done && aim == hit,
        (Gate::Power(aim), Deed::Powered(hit)) => aim == hit,
        (Gate::EndTurn, Deed::Ended) => true,
        _ => false,
    }
}

pub fn gate(tutorial: Tutorial) -> Option<Gate> {
    if !tutorial.active {
        return None;
    }
    LESSONS.get(tutorial.step).map(|lesson| lesson.gate)
}

pub fn focus(tutorial: Tutorial) -> Focus {
    if !tutorial.active {
        return Focus::None;
    }
    LESSONS
        .get(tutorial.step)
        .map_or(Focus::None, |lesson| lesson.focus)
}

pub fn permits(current: Gate, battle: &Battle, selection: Selection, intent: &Intent) -> bool {
    if matches!(intent, Intent::Cancel) {
        return true;
    }
    match current {
        Gate::Next | Gate::EnemyTurn | Gate::Win => false,
        Gate::EndTurn => matches!(intent, Intent::EndTurn),
        Gate::Play(card) => {
            matches!(intent, Intent::Pick(Pick::Hand(id)) if hand_is(battle, *id, card))
        }
        Gate::Cast(card, aim) => match (selection, intent) {
            (Selection::None, Intent::Pick(Pick::Hand(id))) => hand_is(battle, *id, card),
            (Selection::Spell(spell), Intent::Pick(pick)) => {
                *pick == Pick::Hand(spell) || pick_aims(battle, *pick, aim)
            }
            _ => false,
        },
        Gate::Attack(card, aim) => match (selection, intent) {
            (Selection::None, Intent::Pick(Pick::Mine(id))) => mine_is(battle, *id, card),
            (Selection::Attacker(attacker), Intent::Pick(pick)) => {
                *pick == Pick::Mine(attacker) || pick_aims(battle, *pick, aim)
            }
            _ => false,
        },
        Gate::Power(aim) => match (selection, intent) {
            (Selection::None, Intent::Power) => true,
            (Selection::Power, Intent::Pick(pick)) => pick_aims(battle, *pick, aim),
            _ => false,
        },
    }
}

pub fn deed(battle: &Battle, selection: Selection, intent: &Intent) -> Option<Deed> {
    match (selection, intent) {
        (_, Intent::EndTurn) => Some(Deed::Ended),
        (Selection::None, Intent::Pick(Pick::Hand(id))) => {
            let card = battle.player.hand_card(*id)?.card;
            (card.target_rule() == TargetRule::None).then_some(Deed::Played(card))
        }
        (Selection::Spell(spell), Intent::Pick(pick)) => {
            let card = battle.player.hand_card(spell)?.card;
            let aim = aim_of(battle, pick_target(*pick)?)?;
            Some(Deed::Cast(card, aim))
        }
        (Selection::Attacker(attacker), Intent::Pick(pick)) => {
            let card = battle.player.minion(attacker)?.card;
            let aim = aim_of(battle, pick_target(*pick)?)?;
            Some(Deed::Attacked(card, aim))
        }
        (Selection::Power, Intent::Pick(pick)) => {
            Some(Deed::Powered(aim_of(battle, pick_target(*pick)?)?))
        }
        _ => None,
    }
}

fn hand_is(battle: &Battle, id: u32, card: CardId) -> bool {
    battle
        .player
        .hand_card(id)
        .is_some_and(|instance| instance.card == card)
}

fn mine_is(battle: &Battle, id: u32, card: CardId) -> bool {
    battle
        .player
        .minion(id)
        .is_some_and(|minion| minion.card == card)
}

fn pick_aims(battle: &Battle, pick: Pick, aim: Aim) -> bool {
    match pick {
        Pick::Hero(Who::Enemy) => aim == Aim::Warden,
        Pick::Foe(id) => battle
            .enemy
            .minion(id)
            .is_some_and(|minion| Aim::Minion(minion.card) == aim),
        _ => false,
    }
}
