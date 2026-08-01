use crate::data::{Save, SavedFighter, UPGRADES};
use crate::queries::Fighters;
use crate::resources::Campaign;
use crate::theme;
use nightshade::prelude::*;
use nightshade_save::prelude::*;

pub fn write_the_run(campaign: &Campaign, fighters: &mut Fighters<'_, '_>) {
    let held = crate::queries::all_fighters(fighters);
    let slots: Vec<(Entity, usize)> = held
        .iter()
        .map(|(entity, fighter)| (*entity, fighter.slot))
        .collect();
    let mut rows = Vec::new();
    for (_entity, fighter) in &held {
        rows.push(SavedFighter {
            slot: fighter.slot,
            experience: fighter.experience,
            rapport: fighter.rapport,
            morale: fighter.morale,
            wounded: fighter.wounded,
            kills: fighter.kills,
            battles: fighter.battles,
            arc_stage: fighter.arc_stage,
            marks: fighter.marks,
            joined: fighter.joined,
            fallen: fighter.fallen,
            deployed: fighter.deployed,
            taken: fighter
                .taken
                .iter()
                .filter_map(|upgrade| UPGRADES.iter().position(|known| known == upgrade))
                .collect(),
            bonds: fighter
                .bonds
                .iter()
                .filter_map(|(other, points)| {
                    slots
                        .iter()
                        .find(|(entity, _)| entity == other)
                        .map(|(_, slot)| (*slot, *points))
                })
                .collect(),
        });
    }
    let save = Save {
        chapter: campaign.chapter,
        day: campaign.day,
        actions: campaign.actions,
        supplies: campaign.supplies,
        fighters: rows,
    };
    let label = format!("Chapter {}, day {}", save.chapter + 1, save.day);
    save_write(theme::SAVE_SLOT, &label, &save);
}

pub fn read_the_run() -> Option<Save> {
    save_read(theme::SAVE_SLOT)
}

pub fn forget_the_run() {
    save_remove::<Save>(theme::SAVE_SLOT);
}

pub fn restore_the_run(campaign: &mut Campaign, fighters: &mut Fighters<'_, '_>, save: &Save) {
    campaign.chapter = save.chapter;
    campaign.day = save.day;
    campaign.actions = save.actions;
    campaign.supplies = save.supplies;
    campaign.arrival = true;
    campaign.march = false;
    campaign.settled = false;
    campaign.over = false;
    let slots: Vec<(Entity, usize)> = crate::queries::all_fighters(fighters)
        .iter()
        .map(|(entity, fighter)| (*entity, fighter.slot))
        .collect();
    crate::commands::edit_fighters(fighters, |fighter| {
        let Some(row) = save.fighters.iter().find(|row| row.slot == fighter.slot) else {
            return;
        };
        fighter.experience = row.experience;
        fighter.rapport = row.rapport;
        fighter.morale = row.morale;
        fighter.wounded = row.wounded;
        fighter.kills = row.kills;
        fighter.battles = row.battles;
        fighter.arc_stage = row.arc_stage;
        fighter.marks = row.marks;
        fighter.joined = row.joined;
        fighter.fallen = row.fallen;
        fighter.deployed = row.deployed;
        fighter.taken = row
            .taken
            .iter()
            .filter_map(|index| UPGRADES.get(*index).copied())
            .collect();
        fighter.bonds = row
            .bonds
            .iter()
            .filter_map(|(slot, points)| {
                slots
                    .iter()
                    .find(|(_, known)| known == slot)
                    .map(|(entity, _)| (*entity, *points))
            })
            .collect();
        fighter.boon = fighter
            .taken
            .iter()
            .fold(crate::data::Growth::NONE, |total, upgrade| {
                total + upgrade.growth()
            });
    });
}
