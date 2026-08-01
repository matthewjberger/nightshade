use crate::data::{CropKind, Save, SavedPlot};
use crate::resources::{Diorama, Dungeon, Farmland, Hero, Inventory, Quests, Vigor};

pub fn snapshot(
    diorama: &Diorama,
    hour: f32,
    inventory: &Inventory,
    quests: &Quests,
    hero: &Hero,
    vigor: &Vigor,
    dungeon: &Dungeon,
    farmland: &Farmland,
    crops: &[(usize, CropKind, f32)],
) -> Save {
    let plots = farmland
        .plots
        .iter()
        .enumerate()
        .map(|(index, plot)| {
            let growing = crops.iter().find(|(plot_index, _, _)| *plot_index == index);
            SavedPlot {
                column: plot.column,
                row: plot.row,
                watered_day: plot.watered_day,
                crop: growing.map(|(_, kind, _)| kind.index()),
                progress: growing.map_or(0.0, |(_, _, progress)| *progress),
            }
        })
        .collect();
    Save {
        day: diorama.day,
        hour,
        coins: inventory.coins,
        counts: inventory.counts.to_vec(),
        tools: inventory.tools.to_vec(),
        levels: inventory.levels.to_vec(),
        slot: inventory.slot,
        errands: quests.done.to_vec(),
        stamina: vigor.stamina,
        hp: hero.hp,
        max_hp: hero.max_hp,
        xp: hero.xp,
        level: hero.level,
        weapon: hero.weapon.index(),
        deaths: hero.deaths,
        deepest: dungeon.deepest,
        plots,
    }
}
