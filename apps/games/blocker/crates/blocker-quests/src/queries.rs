use crate::data::quests::{Objective, QUESTS};
use crate::resources::Journal;
use blocker_pocket::queries::inventory_count;
use blocker_pocket::resources::Inventory;

pub fn objective_progress(
    objective: &Objective,
    inventory: &Inventory,
    journal: &Journal,
) -> (usize, usize) {
    let (have, want) = match objective {
        Objective::Gather { item, count } => {
            (inventory_count(inventory, *item) as usize, *count as usize)
        }
        Objective::Explore { count } => (journal.discovered.len(), *count),
        Objective::Slay { beast, count } => (journal.slain_count(*beast) as usize, *count as usize),
        Objective::Meet { giver } => (usize::from(journal.met.contains(giver)), 1),
        Objective::Visit { place } => (usize::from(journal.seen.contains(place)), 1),
        Objective::Report => (1, 1),
    };
    (have.min(want), want)
}

pub fn objective_met(objective: &Objective, inventory: &Inventory, journal: &Journal) -> bool {
    let (have, want) = objective_progress(objective, inventory, journal);
    have >= want
}

pub fn xp_for_level(level: u32) -> u32 {
    50 * level.saturating_sub(1) * level
}

pub fn level_from_xp(xp: u32) -> u32 {
    let mut level = 1;
    while xp >= xp_for_level(level + 1) {
        level += 1;
    }
    level
}

pub fn xp_into_level(xp: u32) -> (u32, u32) {
    let level = level_from_xp(xp);
    let base = xp_for_level(level);
    let next = xp_for_level(level + 1);
    (xp - base, next - base)
}

pub fn offered_quest(journal: &Journal, first: usize) -> usize {
    let mut quest = first;
    while journal.finished.contains(&quest) {
        let Some(next) = QUESTS[quest].next else {
            break;
        };
        quest = next;
    }
    quest
}
