use crate::commands::audio::play;
use crate::commands::effects;
use crate::commands::{
    clear_highlights, edit_unit, edit_units, log, look_at, set_phase, show_danger, start_turn,
};
use crate::data::{CHAPTERS, StructureKind, Team};
use crate::queries::{
    Structures, Units, danger_tiles, mission_result, ready_unit, roster, roster_index, site_at,
    sites,
};
use crate::resources::{
    Battle, Battlefield, CameraRig, Campaign, Cursor, Highlights, Phase, Terrain,
};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn tick_the_clocks(
    mut battle: ResMut<Battle>,
    campaign: Res<Campaign>,
    mut highlights: ResMut<Highlights>,
    terrain: Res<Terrain>,
    mut cursor: ResMut<Cursor>,
    mut camera_rig: ResMut<CameraRig>,
    mut units: Units<'_, '_>,
    mut structures: Structures<'_, '_>,
) {
    if battle.phase != Phase::Ticking {
        return;
    }
    let mut roster = roster(&mut units);
    if roster.is_empty() {
        return;
    }
    let sites = sites(&mut structures);

    if battle.outcome.is_some() {
        return;
    }
    let chapter_index = campaign.chapter;
    let chapter = &CHAPTERS[chapter_index.min(CHAPTERS.len() - 1)];
    let turn = battle.turn;
    if let Some(winner) = mission_result(chapter, &roster, &sites, turn) {
        battle.outcome = Some(winner);
        clear_highlights(&mut highlights);
        return;
    }

    let mut ticks = 0;
    let mut ready = ready_unit(&roster);
    while ready.is_none() && ticks < theme::TICK_LIMIT {
        for (_, unit) in &mut roster {
            unit.ct += unit.speed.max(1);
        }
        ticks += 1;
        ready = ready_unit(&roster);
    }
    if ticks > 0 {
        edit_units(&mut units, |entity, unit| {
            if let Some(index) = roster_index(&roster, entity) {
                unit.ct = roster[index].1.ct;
            }
        });
    }
    let Some(entity) = ready else {
        return;
    };
    let Some(index) = roster_index(&roster, entity) else {
        return;
    };
    let unit = roster[index].1.clone();
    let stand = crate::queries::terrain::stand(&terrain, unit.column, unit.row);
    {
        start_turn(&mut battle, entity, &unit);
        log(&mut battle, format!("{}'s turn.", unit.name));
    }
    let danger = battle.danger_shown;
    if danger {
        let tiles = danger_tiles(&terrain, &roster, Team::Player);
        show_danger(&mut highlights, tiles);
    }
    cursor.column = unit.column;
    cursor.row = unit.row;
    look_at(&mut camera_rig, stand);
}

pub fn open_the_turn(
    time: Res<Time>,
    battlefield: Res<Battlefield>,
    mut battle: ResMut<Battle>,
    mut units: Units<'_, '_>,
    mut structures: Structures<'_, '_>,
) {
    if battle.phase != Phase::TurnStart {
        return;
    }
    let Some(active) = battle.active else {
        set_phase(&mut battle, Phase::Ticking);
        return;
    };
    if battle.timer == 0.0 {
        let sites = sites(&mut structures);
        let mut words = Vec::new();
        edit_unit(&mut units, active, |unit| {
            let on_depot = site_at(&sites, unit.column, unit.row).is_some_and(|index| {
                let structure = &sites[index].1;
                structure.kind == StructureKind::Depot && structure.owner == Some(unit.team)
            });
            if on_depot && (unit.hp < unit.max_hp || unit.mp < unit.max_mp) {
                let heal = unit.max_hp * theme::DEPOT_HEAL_PERCENT / 100;
                let supply = unit.max_mp * theme::DEPOT_SUPPLY_PERCENT / 100;
                unit.hp = (unit.hp + heal).min(unit.max_hp);
                unit.mp = (unit.mp + supply).min(unit.max_mp);
                unit.flash = theme::FLASH_SECONDS;
                unit.flash_tint = theme::HEAL_TINT;
                words.push(format!("The depot restores {}.", unit.name));
            }
        });
        for line in words {
            log(&mut battle, line);
        }
    }
    battle.timer += time.delta_time;
    if battle.timer < theme::TURN_START_PAUSE {
        return;
    }
    let mut team = None;
    let mut stunned = false;
    let mut name = "";
    edit_unit(&mut units, active, |unit| {
        team = Some(unit.team);
        stunned = unit.stunned > 0;
        name = unit.name;
    });
    if stunned {
        log(
            &mut battle,
            format!("{name} is stunned and loses the turn."),
        );
        battle.moved = true;
        battle.acted = true;
        set_phase(&mut battle, Phase::TurnEnd);
        return;
    }
    match team {
        Some(Team::Player) if battlefield.auto => set_phase(&mut battle, Phase::Thinking),
        Some(Team::Player) => set_phase(&mut battle, Phase::Menu),
        Some(Team::Enemy) => set_phase(&mut battle, Phase::Thinking),
        None => set_phase(&mut battle, Phase::Ticking),
    }
}

pub fn close_the_turn(
    time: Res<Time>,
    terrain: Res<Terrain>,
    mut battle: ResMut<Battle>,
    mut highlights: ResMut<Highlights>,
    mut commands: Commands,
    mut units: Units<'_, '_>,
    mut transforms: Query<(&mut LocalTransform,)>,
) {
    if battle.phase != Phase::TurnEnd {
        return;
    }
    if battle.timer == 0.0 {
        clear_highlights(&mut highlights);
        if let Some(active) = battle.active {
            let rest = (!battle.moved) as i32 * theme::REST_BONUS
                + (!battle.acted) as i32 * theme::REST_BONUS;
            let mut placement = None;
            edit_unit(&mut units, active, |unit| {
                unit.ct = rest;
                unit.stunned = unit.stunned.saturating_sub(1);
                unit.suppressed = unit.suppressed.saturating_sub(1);
                placement = Some(
                    crate::queries::terrain::stand(&terrain, unit.column, unit.row)
                        + Vec3::new(0.0, unit.mobility.hover(), 0.0),
                );
            });
            if let Some(at) = placement {
                transforms.for_entity(active, |(transform,)| transform.translation = at);
            }
        }
        let mut fallen = Vec::new();
        for (entity, unit) in roster(&mut units) {
            if unit.hp <= 0 {
                let mut position = Vec3::zeros();
                transforms.for_entity(entity, |(transform,)| position = transform.translation);
                fallen.push((entity, unit.name, position));
            }
        }
        for (entity, name, position) in fallen {
            commands.despawn_recursive(entity);
            play(&mut commands, theme::SOUND_DOWN, theme::SOUND_LOUD);
            effects::sparkle(&mut commands, position + Vec3::new(0.0, 0.5, 0.0));
            log(&mut battle, format!("{name} is down."));
        }
    }
    battle.timer += time.delta_time;
    if battle.timer >= theme::TURN_END_PAUSE {
        battle.active = None;
        set_phase(&mut battle, Phase::Ticking);
    }
}
