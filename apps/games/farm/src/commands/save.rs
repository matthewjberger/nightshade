use crate::commands::farming::{sow, till};
use crate::data::{CropKind, Save, Weapon};
use crate::resources::{
    Diorama, Dungeon, Farmland, Hero, Inventory, Quests, Sheet, Terrain, Vigor,
};
use crate::theme;
use nightshade::prelude::*;

fn take<T: Copy + Default>(values: &[T], index: usize) -> T {
    values.get(index).copied().unwrap_or_default()
}

pub fn restore(
    commands: &mut Commands,
    sheet: &Sheet,
    terrain: &Terrain,
    farmland: &mut Farmland,
    inventory: &mut Inventory,
    quests: &mut Quests,
    hero: &mut Hero,
    vigor: &mut Vigor,
    diorama: &mut Diorama,
    dungeon: &mut Dungeon,
    save: &Save,
) {
    diorama.day = save.day;
    inventory.coins = save.coins;
    for (index, count) in inventory.counts.iter_mut().enumerate() {
        *count = take(&save.counts, index);
    }
    for (index, owned) in inventory.tools.iter_mut().enumerate() {
        *owned = save.tools.get(index).copied().unwrap_or(true);
    }
    for (index, level) in inventory.levels.iter_mut().enumerate() {
        *level = take(&save.levels, index);
    }
    inventory.slot = save.slot;
    for (index, done) in quests.done.iter_mut().enumerate() {
        *done = take(&save.errands, index);
    }
    vigor.stamina = save.stamina.clamp(0.0, theme::MAX_STAMINA);
    hero.hp = save.hp.max(1.0);
    hero.max_hp = save.max_hp.max(theme::HERO_MAX_HP);
    hero.xp = save.xp;
    hero.level = save.level.max(1);
    hero.weapon = Weapon::ALL
        .get(save.weapon)
        .copied()
        .unwrap_or(Weapon::Fists);
    hero.deaths = save.deaths;
    dungeon.deepest = save.deepest;
    for plot in &save.plots {
        let index = farmland.plots.len();
        till(commands, terrain, farmland, plot.column, plot.row);
        farmland.plots[index].watered_day = plot.watered_day;
        let Some(kind) = plot.crop.and_then(|kind| CropKind::ALL.get(kind).copied()) else {
            continue;
        };
        sow(
            commands,
            sheet,
            terrain,
            farmland,
            index,
            kind,
            plot.progress.clamp(0.0, 1.0),
        );
    }
}
