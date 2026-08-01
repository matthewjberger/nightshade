pub mod audio;
pub mod camera;
pub mod camp;
pub mod campaign;
pub mod cursor;
pub mod daylight;
pub mod decor;
pub mod digits;
pub mod effects;
pub mod framing;
pub mod interface;
pub mod menus;
pub mod movement;
pub mod save;
pub mod textures;
pub mod units;
pub mod world;

use crate::components::{
    ArcStep, Attack, Avatar, Battles, Bonds, Boon, Busy, Called, Column, Ct, Defense, Deployed,
    Evasion, Experience, Fallen, Flag, Flash, FlashTint, Flipped, Holder, Hp, Joined, Jump, Kills,
    Magic, Marks, MaxHp, MaxMp, Merit, Morale, MoveRange, Mp, Rapport, Resist, Row, Slot, Speed,
    Stunned, Suppressed, Taken, Title, Wounded,
};
use crate::data::{
    ArcStage, Bond, COMPANIONS, Fighter, Growth, Structure, Team, Unit, Upgrade, level_of,
};
use crate::queries::{Fighters, Structures, Units, fighter_row, unit_row};
use crate::resources::{Battle, CameraRig, CampPhase, Campaign, Highlights, Phase};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::{Commands, Entity};

pub fn write_fighter(fighters: &mut Fighters<'_, '_>, entity: Entity, row: &Fighter) {
    fighters.names.for_entity(
        entity,
        |(name, title, class, taste, mood, slot, column, row_index)| {
            name.0 = row.name;
            title.0 = row.title;
            *class = row.class;
            *taste = row.taste;
            *mood = row.mood;
            slot.0 = row.slot;
            column.0 = row.column;
            row_index.0 = row.row;
        },
    );
    fighters.heart.for_entity(
        entity,
        |(facing, experience, rapport, bonds, morale, wounded, deployed, busy)| {
            *facing = row.facing;
            experience.0 = row.experience;
            rapport.0 = row.rapport;
            bonds.0 = row.bonds.clone();
            morale.0 = row.morale;
            wounded.0 = row.wounded;
            deployed.0 = row.deployed;
            busy.0 = row.busy;
        },
    );
    fighters.record.for_entity(
        entity,
        |(kills, battles, joined, arc_step, boon, fallen, marks, taken)| {
            kills.0 = row.kills;
            battles.0 = row.battles;
            joined.0 = row.joined;
            arc_step.0 = row.arc_stage;
            boon.0 = row.boon;
            fallen.0 = row.fallen;
            marks.0 = row.marks;
            taken.0 = row.taken.clone();
        },
    );
}

pub fn edit_fighter(
    fighters: &mut Fighters<'_, '_>,
    entity: Entity,
    change: impl FnOnce(&mut Fighter),
) {
    let Some(mut row) = fighter_row(fighters, entity) else {
        return;
    };
    change(&mut row);
    write_fighter(fighters, entity, &row);
}

pub fn edit_fighters(fighters: &mut Fighters<'_, '_>, mut change: impl FnMut(&mut Fighter)) {
    let mut held = Vec::new();
    fighters.names.for_each_mut(|entity, _| held.push(entity));
    for entity in held {
        edit_fighter(fighters, entity, &mut change);
    }
}

pub fn set_fighter(commands: &mut Commands, entity: Entity, row: &Fighter) {
    commands.set(entity, Called(row.name));
    commands.set(entity, Title(row.title));
    commands.set(entity, row.class);
    commands.set(entity, row.taste);
    commands.set(entity, row.mood);
    commands.set(entity, Slot(row.slot));
    commands.set(entity, Column(row.column));
    commands.set(entity, Row(row.row));
    commands.set(entity, row.facing);
    commands.set(entity, Experience(row.experience));
    commands.set(entity, Rapport(row.rapport));
    commands.set(entity, Bonds(row.bonds.clone()));
    commands.set(entity, Morale(row.morale));
    commands.set(entity, Wounded(row.wounded));
    commands.set(entity, Deployed(row.deployed));
    commands.set(entity, Busy(row.busy));
    commands.set(entity, Kills(row.kills));
    commands.set(entity, Battles(row.battles));
    commands.set(entity, Joined(row.joined));
    commands.set(entity, ArcStep(row.arc_stage));
    commands.set(entity, Boon(row.boon));
    commands.set(entity, Fallen(row.fallen));
    commands.set(entity, Marks(row.marks));
    commands.set(entity, Taken(row.taken.clone()));
}

pub fn write_unit(units: &mut Units<'_, '_>, entity: Entity, row: &Unit) {
    units.who.for_entity(
        entity,
        |(avatar, bonds, merit, kills, name, class, team, mobility)| {
            avatar.0 = row.fighter;
            bonds.0 = row.bonds.clone();
            merit.0 = row.merit;
            kills.0 = row.kills;
            name.0 = row.name;
            *class = row.class;
            *team = row.team;
            *mobility = row.mobility;
        },
    );
    units.vitals.for_entity(
        entity,
        |(hp, max_hp, mp, max_mp, attack, defense, magic, resist)| {
            hp.0 = row.hp;
            max_hp.0 = row.max_hp;
            mp.0 = row.mp;
            max_mp.0 = row.max_mp;
            attack.0 = row.attack;
            defense.0 = row.defense;
            magic.0 = row.magic;
            resist.0 = row.resist;
        },
    );
    units.grades.for_entity(
        entity,
        |(speed, move_range, jump, evasion, ct, flipped, flash, flash_tint)| {
            speed.0 = row.speed;
            move_range.0 = row.move_range;
            jump.0 = row.jump;
            evasion.0 = row.evasion;
            ct.0 = row.ct;
            flipped.0 = row.flipped;
            flash.0 = row.flash;
            flash_tint.0 = row.flash_tint;
        },
    );
    units.stance.for_entity(
        entity,
        |(facing, column, row_index, stunned, suppressed)| {
            *facing = row.facing;
            column.0 = row.column;
            row_index.0 = row.row;
            stunned.0 = row.stunned;
            suppressed.0 = row.suppressed;
        },
    );
}

pub fn edit_unit(units: &mut Units<'_, '_>, entity: Entity, change: impl FnOnce(&mut Unit)) {
    let Some(mut row) = unit_row(units, entity) else {
        return;
    };
    change(&mut row);
    write_unit(units, entity, &row);
}

pub fn edit_units(units: &mut Units<'_, '_>, mut change: impl FnMut(Entity, &mut Unit)) {
    let mut held = Vec::new();
    units.who.for_each_mut(|entity, _| held.push(entity));
    for entity in held {
        edit_unit(units, entity, |row| change(entity, row));
    }
}

pub fn set_unit(commands: &mut Commands, entity: Entity, row: &Unit) {
    commands.set(entity, Avatar(row.fighter));
    commands.set(entity, Bonds(row.bonds.clone()));
    commands.set(entity, Merit(row.merit));
    commands.set(entity, Kills(row.kills));
    commands.set(entity, Called(row.name));
    commands.set(entity, row.class);
    commands.set(entity, row.team);
    commands.set(entity, row.mobility);
    commands.set(entity, Hp(row.hp));
    commands.set(entity, MaxHp(row.max_hp));
    commands.set(entity, Mp(row.mp));
    commands.set(entity, MaxMp(row.max_mp));
    commands.set(entity, Attack(row.attack));
    commands.set(entity, Defense(row.defense));
    commands.set(entity, Magic(row.magic));
    commands.set(entity, Resist(row.resist));
    commands.set(entity, Speed(row.speed));
    commands.set(entity, MoveRange(row.move_range));
    commands.set(entity, Jump(row.jump));
    commands.set(entity, Evasion(row.evasion));
    commands.set(entity, Ct(row.ct));
    commands.set(entity, row.facing);
    commands.set(entity, Column(row.column));
    commands.set(entity, Row(row.row));
    commands.set(entity, Flipped(row.flipped));
    commands.set(entity, Flash(row.flash));
    commands.set(entity, FlashTint(row.flash_tint));
    commands.set(entity, Stunned(row.stunned));
    commands.set(entity, Suppressed(row.suppressed));
}

pub fn set_structure(commands: &mut Commands, entity: Entity, row: &Structure) {
    commands.set(entity, row.kind);
    commands.set(entity, Holder(row.owner));
    commands.set(entity, Column(row.column));
    commands.set(entity, Row(row.row));
    commands.set(entity, Flag(row.flag));
}

pub fn edit_structure(
    structures: &mut Structures<'_, '_>,
    entity: Entity,
    change: impl FnOnce(&mut Structure),
) {
    let Some(mut row) = crate::queries::structure_row(structures, entity) else {
        return;
    };
    change(&mut row);
    structures.for_entity(entity, |(kind, holder, column, row_index, flag)| {
        *kind = row.kind;
        holder.0 = row.owner;
        column.0 = row.column;
        row_index.0 = row.row;
        flag.0 = row.flag;
    });
}

pub fn story(campaign: &mut Campaign, text: impl Into<String>) {
    let line = text.into();
    if campaign.story.last() == Some(&line) {
        return;
    }
    campaign.story.push(line);
    let keep = theme::LOG_LINES * 4;
    if campaign.story.len() > keep {
        let drop = campaign.story.len() - keep;
        campaign.story.drain(..drop);
    }
}

pub fn speak(campaign: &mut Campaign, lines: Vec<(String, String)>) {
    if lines.is_empty() {
        finish_the_speech(campaign);
        return;
    }
    for (speaker, line) in &lines {
        story(campaign, format!("{speaker}: {line}"));
    }
    campaign.speech = lines;
    campaign.said = 0;
    campaign.phase = CampPhase::Speaking;
}

pub fn begin_the_run(campaign: &mut Campaign) {
    campaign.supplies = theme::START_SUPPLIES;
    campaign.story.clear();
    campaign.report.clear();
    open_chapter(campaign, 0);
}

pub fn enlist_again(fighter: &mut Fighter) {
    fighter.experience = 0;
    fighter.rapport = 0;
    fighter.bonds.clear();
    fighter.morale = theme::START_MORALE;
    fighter.wounded = 0;
    fighter.busy = false;
    fighter.kills = 0;
    fighter.battles = 0;
    fighter.joined = false;
    fighter.arc_stage = 0;
    fighter.boon = Growth::default();
}

pub fn finish_the_speech(campaign: &mut Campaign) {
    let Some(entity) = campaign.greeted.take() else {
        set_camp_phase(campaign, CampPhase::Roam);
        return;
    };
    set_camp_phase(campaign, CampPhase::Menu);
    campaign.selected = Some(entity);
    campaign.menu_index = 0;
}

pub fn set_camp_phase(campaign: &mut Campaign, phase: CampPhase) {
    campaign.phase = phase;
    if phase != CampPhase::Partner {
        campaign.partner_of = None;
        campaign.activity = None;
    }
    if phase == CampPhase::Roam {
        campaign.selected = None;
    }
    if phase != CampPhase::Speaking {
        campaign.speech.clear();
        campaign.said = 0;
    }
}

pub fn spend(campaign: &mut Campaign, actions: u32, supplies: i32) {
    campaign.actions = campaign.actions.saturating_sub(actions);
    campaign.supplies -= supplies;
}

pub fn open_chapter(campaign: &mut Campaign, chapter: usize) {
    campaign.keep = true;
    campaign.chapter = chapter;
    campaign.day = 1;
    campaign.actions = theme::ACTIONS_PER_DAY;
    campaign.menu_index = 0;
    campaign.march = false;
    campaign.settled = false;
    campaign.arrival = true;
    campaign.briefed = false;
    campaign.greeted = None;
    set_camp_phase(campaign, CampPhase::Roam);
}

pub fn advance_day(campaign: &mut Campaign) {
    campaign.keep = true;
    campaign.day += 1;
    campaign.actions = theme::ACTIONS_PER_DAY;
}

pub fn raise_rapport(fighter: &mut Fighter, points: u32) -> Vec<&'static ArcStage> {
    fighter.rapport += points;
    fighter.busy = true;
    advance_the_arc(fighter)
}

fn advance_the_arc(fighter: &mut Fighter) -> Vec<&'static ArcStage> {
    let rank = Bond::of(fighter.rapport).rank();
    let companion = &COMPANIONS[fighter.slot.min(COMPANIONS.len() - 1)];
    let mut opened = Vec::new();
    while fighter.arc_stage < rank {
        let stage = &companion.stages[fighter.arc_stage.min(3) as usize];
        fighter.arc_stage += 1;
        fighter.boon = fighter.boon + stage.boon;
        opened.push(stage);
    }
    opened
}

pub fn raise_bond(fighter: &mut Fighter, other: Entity, points: u32) {
    match fighter
        .bonds
        .iter_mut()
        .find(|(entity, _)| *entity == other)
    {
        Some((_, held)) => *held += points,
        None => fighter.bonds.push((other, points)),
    }
}

pub fn add_experience(fighter: &mut Fighter, points: u32) {
    let before = level_of(fighter.experience);
    fighter.experience += points;
    let after = level_of(fighter.experience);
    fighter.marks += after.saturating_sub(before);
}

pub fn take_the_upgrade(fighter: &mut Fighter, upgrade: Upgrade) {
    fighter.boon = fighter.boon + upgrade.growth();
    fighter.taken.push(upgrade);
    fighter.marks = fighter.marks.saturating_sub(1);
}

pub fn lift_morale(fighter: &mut Fighter, amount: i32) {
    fighter.morale = (fighter.morale + amount).clamp(0, theme::MORALE_MAX);
}

pub fn wound(fighter: &mut Fighter, days: u32) {
    fighter.wounded = fighter.wounded.max(days);
    fighter.deployed = false;
    lift_morale(fighter, theme::MORALE_WOUND);
}

pub fn wake_the_camp(fighter: &mut Fighter) {
    fighter.busy = false;
    fighter.wounded = fighter.wounded.saturating_sub(1);
}

pub fn log(battle: &mut Battle, text: impl Into<String>) {
    battle.log.push(text.into());
    let keep = theme::LOG_LINES * 4;
    if battle.log.len() > keep {
        let drop = battle.log.len() - keep;
        battle.log.drain(..drop);
    }
}

pub fn set_phase(battle: &mut Battle, phase: Phase) {
    battle.phase = phase;
    battle.timer = 0.0;
}

pub fn start_turn(battle: &mut Battle, entity: Entity, unit: &Unit) {
    battle.active = Some(entity);
    battle.moved = false;
    battle.acted = false;
    battle.menu_index = 0;
    battle.ability_index = 0;
    battle.reachable.clear();
    battle.targets.clear();
    battle.ability = None;
    battle.target_tile = None;
    battle.path.clear();
    battle.plan = None;
    battle.hits.clear();
    battle.hits_rolled = false;
    battle.hits_applied = false;
    battle.counters_applied = false;
    battle.start_column = unit.column;
    battle.start_row = unit.row;
    battle.start_facing = unit.facing;
    battle.turn += 1;
    set_phase(battle, Phase::TurnStart);
}

pub fn reset_battle(battle: &mut Battle) {
    let rng = battle.rng.clone();
    let danger_shown = battle.danger_shown;
    *battle = Battle {
        rng,
        danger_shown,
        ..Default::default()
    };
}

pub fn show_moves(highlights: &mut Highlights, tiles: Vec<(u32, u32)>) {
    highlights.move_tiles = tiles;
    highlights.target_tiles.clear();
    highlights.area_tiles.clear();
    highlights.path_tiles.clear();
    highlights.version += 1;
}

pub fn show_path(highlights: &mut Highlights, tiles: Vec<(u32, u32)>) {
    if highlights.path_tiles != tiles {
        highlights.path_tiles = tiles;
        highlights.version += 1;
    }
}

pub fn show_targets(highlights: &mut Highlights, tiles: Vec<(u32, u32)>) {
    highlights.move_tiles.clear();
    highlights.path_tiles.clear();
    highlights.target_tiles = tiles;
    highlights.area_tiles.clear();
    highlights.version += 1;
}

pub fn show_area(highlights: &mut Highlights, tiles: Vec<(u32, u32)>) {
    if highlights.area_tiles != tiles {
        highlights.area_tiles = tiles;
        highlights.version += 1;
    }
}

pub fn show_danger(highlights: &mut Highlights, tiles: Vec<(u32, u32)>) {
    if highlights.danger_tiles != tiles {
        highlights.danger_tiles = tiles;
        highlights.version += 1;
    }
}

pub fn clear_highlights(highlights: &mut Highlights) {
    let empty = highlights.move_tiles.is_empty()
        && highlights.target_tiles.is_empty()
        && highlights.area_tiles.is_empty()
        && highlights.path_tiles.is_empty();
    if empty {
        return;
    }
    highlights.move_tiles.clear();
    highlights.target_tiles.clear();
    highlights.area_tiles.clear();
    highlights.path_tiles.clear();
    highlights.version += 1;
}

pub fn capture(structure: &mut Structure, team: Team) {
    structure.owner = Some(team);
}

pub fn look_at(rig: &mut CameraRig, target: Vec3) {
    rig.target = target;
}

pub fn turn_rig(rig: &mut CameraRig, steps: i32) {
    rig.yaw_index += steps;
}

pub fn zoom_rig(rig: &mut CameraRig, steps: i32) {
    let count = theme::CAMERA_DISTANCES.len() as i32;
    rig.distance_index = (rig.distance_index as i32 + steps).clamp(0, count - 1) as usize;
}

pub fn tilt_rig(rig: &mut CameraRig) {
    rig.pitch_index = (rig.pitch_index + 1) % theme::CAMERA_PITCHES.len();
}
