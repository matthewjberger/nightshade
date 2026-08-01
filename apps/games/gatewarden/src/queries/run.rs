use crate::data::cards::{CardId, Kind};
use crate::data::chapters::{chapter_scene, chapter_won, story_end};
use crate::data::relics::Relic;
use crate::data::shop::{DECK_MAX, GATE_COUNT, Offer, RELIC_SLOTS, STRIKE_FLOOR};
use crate::data::story::{
    Ending, ForkOutcome, MERCY_FOR_OPEN_GATES, SceneText, THIN_COUNT, arrival, ending, prologue,
    victory, yard, yard_end,
};
use crate::resources::{Run, SceneKind};

pub fn scene_text(run: &Run) -> SceneText {
    match run.scene {
        SceneKind::Prologue => prologue(),
        SceneKind::Arrival(gate) => arrival(gate),
        SceneKind::Victory { gate, spared } => victory(gate, spared),
        SceneKind::Ending(kind) => ending(kind),
        SceneKind::Yard => yard(),
        SceneKind::YardEnd => yard_end(),
        SceneKind::Chapter(index) => chapter_scene(index),
        SceneKind::ChapterWon(index) => chapter_won(index),
        SceneKind::StoryEnd => story_end(),
    }
}

pub fn can_choose(run: &Run, outcome: ForkOutcome) -> bool {
    match outcome {
        ForkOutcome::Tithe { gold } => run.gold >= gold,
        ForkOutcome::Thin => run.deck.len() > THIN_COUNT + 4,
        ForkOutcome::Duel | ForkOutcome::Borrow | ForkOutcome::Spare => true,
    }
}

pub fn can_buy(run: &Run, offer: Offer) -> bool {
    run.gold >= offer.cost()
        && match offer {
            Offer::Card(_) => run.deck.len() < DECK_MAX,
            Offer::Relic(_) => run.relics.len() < RELIC_SLOTS,
            Offer::Forge(card) => run.deck.contains(&card),
            Offer::Purge(card) => run.deck.contains(&card) && run.deck.len() > STRIKE_FLOOR,
        }
}

pub fn boon_for(best_gate: u32) -> Option<Relic> {
    match best_gate {
        0..=2 => None,
        3..=4 => Some(Relic::WardingCharm),
        5..=6 => Some(Relic::Whetstone),
        _ => Some(Relic::ThickHide),
    }
}

pub fn strike_target(run: &Run) -> Option<CardId> {
    if run.deck.len() <= STRIKE_FLOOR {
        return None;
    }
    run.deck
        .iter()
        .copied()
        .min_by_key(|card| match card.def().kind {
            Kind::Minion { attack, health, .. } => attack + health,
            Kind::Spell { .. } => i32::MAX,
        })
}

pub fn can_strike(run: &Run) -> bool {
    run.gold >= run.strike_cost && strike_target(run).is_some()
}

pub fn gate_pick(run: &Run, gate: u32) -> u8 {
    let index = gate.clamp(1, GATE_COUNT) as usize - 1;
    run.roster[index]
}

pub fn decide_ending(shards: usize, mercy: u32) -> Ending {
    if shards >= 7 {
        Ending::Whole
    } else if mercy >= MERCY_FOR_OPEN_GATES {
        Ending::OpenGates
    } else {
        Ending::Summit
    }
}

pub fn warden_pick(run: &Run) -> u8 {
    let index = run.gate.clamp(1, GATE_COUNT) as usize - 1;
    run.roster[index]
}
