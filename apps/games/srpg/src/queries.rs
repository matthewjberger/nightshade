pub mod camp;
pub mod field;
pub mod menu;
pub mod rig;
pub mod sheet;
pub mod terrain;

use crate::components::{
    ArcStep, Attack, Avatar, Battles, Bonds, Boon, Busy, Called, Column, Ct, Defense, Deployed,
    Evasion, Experience, Fallen, Flag, Flash, FlashTint, Flipped, Holder, Hp, Joined, Jump, Kills,
    Magic, Marks, MaxHp, MaxMp, Merit, Morale, MoveRange, Mp, Rapport, Resist, Row, Slot, Speed,
    Stunned, Suppressed, Taken, Title, Wounded,
};
use crate::data::{
    Ability, Ask, Bond, Chapter, Class, Effect, Facing, Fighter, Mobility, Mood, Objective, Side,
    Strike, StrikeInput, Structure, StructureKind, Taste, Team, Unit, side_of_attack, strike,
};
use crate::resources::{Phase, Plan, Reach, Screen, SrpgArguments, Terrain};
use crate::theme;
use nightshade::prelude::Entity;
use nightshade_input::Actions;
use nightshade_picking::PickingRay;
use std::collections::VecDeque;

pub type Roster = Vec<(Entity, Unit)>;
pub type Sites = Vec<(Entity, Structure)>;
pub type Squad = Vec<(Entity, Fighter)>;

type Query<'world, T> = nightshade::ecs::system_param::Query<'world, T>;

pub type FighterNames<'world, 'item> = Query<
    'world,
    (
        &'item mut Called,
        &'item mut Title,
        &'item mut Class,
        &'item mut Taste,
        &'item mut Mood,
        &'item mut Slot,
        &'item mut Column,
        &'item mut Row,
    ),
>;

pub type FighterHeart<'world, 'item> = Query<
    'world,
    (
        &'item mut Facing,
        &'item mut Experience,
        &'item mut Rapport,
        &'item mut Bonds,
        &'item mut Morale,
        &'item mut Wounded,
        &'item mut Deployed,
        &'item mut Busy,
    ),
>;

pub type FighterRecord<'world, 'item> = Query<
    'world,
    (
        &'item mut Kills,
        &'item mut Battles,
        &'item mut Joined,
        &'item mut ArcStep,
        &'item mut Boon,
        &'item mut Fallen,
        &'item mut Marks,
        &'item mut Taken,
    ),
>;

nightshade::ecs::query_bundle! {
    pub struct Fighters<'world, 'item> {
        pub names: FighterNames<'world, 'item>,
        pub heart: FighterHeart<'world, 'item>,
        pub record: FighterRecord<'world, 'item>,
    }
}

pub type UnitWho<'world, 'item> = Query<
    'world,
    (
        &'item mut Avatar,
        &'item mut Bonds,
        &'item mut Merit,
        &'item mut Kills,
        &'item mut Called,
        &'item mut Class,
        &'item mut Team,
        &'item mut Mobility,
    ),
>;

pub type UnitVitals<'world, 'item> = Query<
    'world,
    (
        &'item mut Hp,
        &'item mut MaxHp,
        &'item mut Mp,
        &'item mut MaxMp,
        &'item mut Attack,
        &'item mut Defense,
        &'item mut Magic,
        &'item mut Resist,
    ),
>;

pub type UnitGrades<'world, 'item> = Query<
    'world,
    (
        &'item mut Speed,
        &'item mut MoveRange,
        &'item mut Jump,
        &'item mut Evasion,
        &'item mut Ct,
        &'item mut Flipped,
        &'item mut Flash,
        &'item mut FlashTint,
    ),
>;

pub type UnitStance<'world, 'item> = Query<
    'world,
    (
        &'item mut Facing,
        &'item mut Column,
        &'item mut Row,
        &'item mut Stunned,
        &'item mut Suppressed,
    ),
>;

nightshade::ecs::query_bundle! {
    pub struct Units<'world, 'item> {
        pub who: UnitWho<'world, 'item>,
        pub vitals: UnitVitals<'world, 'item>,
        pub grades: UnitGrades<'world, 'item>,
        pub stance: UnitStance<'world, 'item>,
    }
}

pub type Structures<'world, 'item> = Query<
    'world,
    (
        &'item mut StructureKind,
        &'item mut Holder,
        &'item mut Column,
        &'item mut Row,
        &'item mut Flag,
    ),
>;

pub fn fighter_row(fighters: &mut Fighters<'_, '_>, entity: Entity) -> Option<Fighter> {
    let mut row = Fighter::default();
    let mut found = false;
    fighters.names.for_entity(
        entity,
        |(name, title, class, taste, mood, slot, column, row_index)| {
            found = true;
            row.name = name.0;
            row.title = title.0;
            row.class = *class;
            row.taste = *taste;
            row.mood = *mood;
            row.slot = slot.0;
            row.column = column.0;
            row.row = row_index.0;
        },
    );
    if !found {
        return None;
    }
    fighters.heart.for_entity(
        entity,
        |(facing, experience, rapport, bonds, morale, wounded, deployed, busy)| {
            row.facing = *facing;
            row.experience = experience.0;
            row.rapport = rapport.0;
            row.bonds = bonds.0.clone();
            row.morale = morale.0;
            row.wounded = wounded.0;
            row.deployed = deployed.0;
            row.busy = busy.0;
        },
    );
    fighters.record.for_entity(
        entity,
        |(kills, battles, joined, arc_stage, boon, fallen, marks, taken)| {
            row.kills = kills.0;
            row.battles = battles.0;
            row.joined = joined.0;
            row.arc_stage = arc_stage.0;
            row.boon = boon.0;
            row.fallen = fallen.0;
            row.marks = marks.0;
            row.taken = taken.0.clone();
        },
    );
    Some(row)
}

pub fn all_fighters(fighters: &mut Fighters<'_, '_>) -> Squad {
    let mut held = Vec::new();
    fighters.names.for_each_mut(|entity, _| held.push(entity));
    let mut rows = Vec::new();
    for entity in held {
        if let Some(row) = fighter_row(fighters, entity) {
            rows.push((entity, row));
        }
    }
    rows
}

pub fn squad(fighters: &mut Fighters<'_, '_>) -> Squad {
    let mut rows: Squad = all_fighters(fighters)
        .into_iter()
        .filter(|(_, fighter)| fighter.joined && !fighter.fallen)
        .collect();
    rows.sort_by_key(|(_, fighter)| fighter.slot);
    rows
}

pub fn mustering(fighters: &mut Fighters<'_, '_>) -> Squad {
    squad(fighters)
}

pub fn unit_row(units: &mut Units<'_, '_>, entity: Entity) -> Option<Unit> {
    let mut row = Unit::default();
    let mut found = false;
    units.who.for_entity(
        entity,
        |(avatar, bonds, merit, kills, name, class, team, mobility)| {
            found = true;
            row.fighter = avatar.0;
            row.bonds = bonds.0.clone();
            row.merit = merit.0;
            row.kills = kills.0;
            row.name = name.0;
            row.class = *class;
            row.team = *team;
            row.mobility = *mobility;
        },
    );
    if !found {
        return None;
    }
    units.vitals.for_entity(
        entity,
        |(hp, max_hp, mp, max_mp, attack, defense, magic, resist)| {
            row.hp = hp.0;
            row.max_hp = max_hp.0;
            row.mp = mp.0;
            row.max_mp = max_mp.0;
            row.attack = attack.0;
            row.defense = defense.0;
            row.magic = magic.0;
            row.resist = resist.0;
        },
    );
    units.grades.for_entity(
        entity,
        |(speed, move_range, jump, evasion, ct, flipped, flash, flash_tint)| {
            row.speed = speed.0;
            row.move_range = move_range.0;
            row.jump = jump.0;
            row.evasion = evasion.0;
            row.ct = ct.0;
            row.flipped = flipped.0;
            row.flash = flash.0;
            row.flash_tint = flash_tint.0;
        },
    );
    units.stance.for_entity(
        entity,
        |(facing, column, row_index, stunned, suppressed)| {
            row.facing = *facing;
            row.column = column.0;
            row.row = row_index.0;
            row.stunned = stunned.0;
            row.suppressed = suppressed.0;
        },
    );
    Some(row)
}

pub fn structure_row(structures: &mut Structures<'_, '_>, entity: Entity) -> Option<Structure> {
    let mut row = Structure::default();
    let mut found = false;
    structures.for_entity(entity, |(kind, holder, column, row_index, flag)| {
        found = true;
        row.kind = *kind;
        row.owner = holder.0;
        row.column = column.0;
        row.row = row_index.0;
        row.flag = flag.0;
    });
    found.then_some(row)
}

pub fn fighter_index(squad: &[(Entity, Fighter)], entity: Entity) -> Option<usize> {
    squad.iter().position(|(candidate, _)| *candidate == entity)
}

pub fn bond_points(bonds: &[(Entity, u32)], other: Entity) -> u32 {
    bonds
        .iter()
        .find(|(entity, _)| *entity == other)
        .map_or(0, |(_, points)| *points)
}

pub fn closest_bond(squad: &[(Entity, Fighter)], index: usize) -> Option<(usize, u32)> {
    squad[index]
        .1
        .bonds
        .iter()
        .filter_map(|(entity, points)| fighter_index(squad, *entity).map(|other| (other, *points)))
        .max_by_key(|(_, points)| *points)
}

pub fn open_tile(terrain: &Terrain, column: u32, row: u32, taken: &[(u32, u32)]) -> (u32, u32) {
    if free_tile(terrain, column, row, taken) {
        return (column, row);
    }
    for ring in 1..theme::SPOT_SEARCH {
        let mut best: Option<((u32, u32), i32)> = None;
        for step in 0..(ring * 8) {
            let (across, down) = ring_step(ring, step);
            let near_column = column as i32 + across;
            let near_row = row as i32 + down;
            if near_column < 0 || near_row < 0 {
                continue;
            }
            let (near_column, near_row) = (near_column as u32, near_row as u32);
            if !free_tile(terrain, near_column, near_row, taken) {
                continue;
            }
            let reach = across * across + down * down;
            if best.is_none_or(|(_, held)| reach < held) {
                best = Some(((near_column, near_row), reach));
            }
        }
        if let Some((tile, _)) = best {
            return tile;
        }
    }
    (column, row)
}

fn free_tile(terrain: &Terrain, column: u32, row: u32, taken: &[(u32, u32)]) -> bool {
    column < terrain.columns
        && row < terrain.rows
        && crate::queries::terrain::can_stop(terrain, column, row, Mobility::Foot)
        && !taken.contains(&(column, row))
}

fn ring_step(ring: i32, step: i32) -> (i32, i32) {
    let side = ring * 2;
    let index = step % (side * 4);
    match index / side {
        0 => (-ring + index % side, -ring),
        1 => (ring, -ring + index % side),
        2 => (ring - index % side, ring),
        _ => (-ring, ring - index % side),
    }
}

pub fn ready_to_deploy(fighter: &Fighter) -> bool {
    fighter.wounded == 0
}

pub fn mustered(squad: &[(Entity, Fighter)]) -> usize {
    squad.iter().filter(|(_, fighter)| fighter.deployed).count()
}

pub fn support_rank(roster: &[(Entity, Unit)], unit: &Unit, tile: (u32, u32)) -> u32 {
    roster
        .iter()
        .filter(|(_, other)| other.team == unit.team)
        .filter(|(_, other)| manhattan(tile, (other.column, other.row)) == 1)
        .filter_map(|(_, other)| other.fighter)
        .map(|fighter| Bond::of(bond_points(&unit.bonds, fighter)).rank())
        .max()
        .unwrap_or(0)
}

pub fn mission_result(
    chapter: &Chapter,
    roster: &[(Entity, Unit)],
    sites: &[(Entity, Structure)],
    turn: u32,
) -> Option<Team> {
    if living(roster, Team::Player) == 0 || !headquarters_held(sites, Team::Player) {
        return Some(Team::Enemy);
    }
    let routed = living(roster, Team::Enemy) == 0;
    let won = match chapter.objective {
        Objective::Rout => routed,
        Objective::Seize => routed || !headquarters_held(sites, Team::Enemy),
        Objective::Survive(turns) => routed || turn > turns,
    };
    won.then_some(Team::Player)
}

pub fn roster(units: &mut Units<'_, '_>) -> Roster {
    let mut held = Vec::new();
    units.who.for_each_mut(|entity, _| held.push(entity));
    let mut rows = Vec::new();
    for entity in held {
        if let Some(row) = unit_row(units, entity) {
            rows.push((entity, row));
        }
    }
    rows
}

pub fn sites(structures: &mut Structures<'_, '_>) -> Sites {
    let mut held = Vec::new();
    structures.for_each_mut(|entity, _| held.push(entity));
    let mut rows = Vec::new();
    for entity in held {
        if let Some(row) = structure_row(structures, entity) {
            rows.push((entity, row));
        }
    }
    rows
}

pub fn facing_toward(from: (u32, u32), to: (u32, u32)) -> Option<Facing> {
    Facing::from_delta(to.0 as i32 - from.0 as i32, to.1 as i32 - from.1 as i32)
}

pub fn cursor_free(phase: Phase) -> bool {
    matches!(phase, Phase::ChooseMove | Phase::ChooseTarget)
}

pub fn cursor_shown(phase: Phase) -> bool {
    matches!(
        phase,
        Phase::Menu | Phase::ChooseMove | Phase::ChooseAbility | Phase::ChooseTarget
    )
}

pub fn unit_at(roster: &[(Entity, Unit)], column: u32, row: u32) -> Option<usize> {
    roster
        .iter()
        .position(|(_, unit)| unit.column == column && unit.row == row)
}

pub fn roster_index(roster: &[(Entity, Unit)], entity: Entity) -> Option<usize> {
    roster
        .iter()
        .position(|(candidate, _)| *candidate == entity)
}

pub fn site_at(sites: &[(Entity, Structure)], column: u32, row: u32) -> Option<usize> {
    sites
        .iter()
        .position(|(_, structure)| structure.column == column && structure.row == row)
}

pub fn manhattan(a: (u32, u32), b: (u32, u32)) -> u32 {
    a.0.abs_diff(b.0) + a.1.abs_diff(b.1)
}

pub fn effective_move(unit: &Unit) -> u32 {
    if unit.suppressed > 0 {
        unit.move_range.div_ceil(2)
    } else {
        unit.move_range
    }
}

pub fn is_air(unit: &Unit) -> bool {
    unit.mobility == Mobility::Air
}

pub fn reachable(terrain: &Terrain, roster: &[(Entity, Unit)], mover: &Unit) -> Vec<Reach> {
    let mut best: Vec<Option<Reach>> = vec![None; terrain.heights.len()];
    let start = (mover.column, mover.row);
    let range = effective_move(mover);
    best[crate::queries::terrain::index(terrain, start.0, start.1)] = Some(Reach {
        column: start.0,
        row: start.1,
        steps: 0,
        from: None,
    });
    let mut queue = VecDeque::from([start]);
    while let Some((column, row)) = queue.pop_front() {
        let Some(here) = best[crate::queries::terrain::index(terrain, column, row)] else {
            continue;
        };
        if here.steps >= range {
            continue;
        }
        let height = crate::queries::terrain::height(terrain, column, row) as i32;
        for (column_delta, row_delta) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let next_column = column as i32 + column_delta;
            let next_row = row as i32 + row_delta;
            if !crate::queries::terrain::contains(terrain, next_column, next_row) {
                continue;
            }
            let (next_column, next_row) = (next_column as u32, next_row as u32);
            if !crate::queries::terrain::passable(terrain, next_column, next_row, mover.mobility) {
                continue;
            }
            let rise = (crate::queries::terrain::height(terrain, next_column, next_row) as i32
                - height)
                .abs();
            if mover.mobility != Mobility::Air && rise > mover.jump as i32 {
                continue;
            }
            if let Some(occupant) = unit_at(roster, next_column, next_row)
                && roster[occupant].1.team != mover.team
            {
                continue;
            }
            let slot = &mut best[crate::queries::terrain::index(terrain, next_column, next_row)];
            if slot.is_some() {
                continue;
            }
            *slot = Some(Reach {
                column: next_column,
                row: next_row,
                steps: here.steps + 1,
                from: Some((column, row)),
            });
            queue.push_back((next_column, next_row));
        }
    }
    best.into_iter().flatten().collect()
}

pub fn destinations(
    terrain: &Terrain,
    reachable: &[Reach],
    roster: &[(Entity, Unit)],
    mover: &Unit,
) -> Vec<(u32, u32)> {
    reachable
        .iter()
        .filter(|reach| {
            crate::queries::terrain::can_stop(terrain, reach.column, reach.row, mover.mobility)
        })
        .filter(|reach| {
            (reach.column == mover.column && reach.row == mover.row)
                || unit_at(roster, reach.column, reach.row).is_none()
        })
        .map(|reach| (reach.column, reach.row))
        .collect()
}

pub fn path_to(reachable: &[Reach], destination: (u32, u32)) -> Vec<(u32, u32)> {
    let mut path = Vec::new();
    let mut cursor = Some(destination);
    while let Some(tile) = cursor {
        let Some(reach) = reachable
            .iter()
            .find(|reach| reach.column == tile.0 && reach.row == tile.1)
        else {
            return Vec::new();
        };
        path.push(tile);
        cursor = reach.from;
    }
    path.reverse();
    path
}

pub fn ability_range(unit: &Unit, ability: Ability) -> (u32, u32, u32) {
    let shape = ability.shape();
    if shape.uses_class_range {
        unit.class.sheet().attack_range
    } else {
        (shape.range_min, shape.range_max, shape.vertical)
    }
}

pub fn air_factor(unit: &Unit, ability: Ability) -> f32 {
    let shape = ability.shape();
    if shape.uses_class_range {
        unit.class.sheet().attack_air
    } else {
        shape.air_factor
    }
}

pub fn is_stationary(unit: &Unit, ability: Ability) -> bool {
    let shape = ability.shape();
    if shape.uses_class_range {
        unit.class.sheet().attack_stationary
    } else {
        shape.stationary
    }
}

pub fn range_tiles(
    terrain: &Terrain,
    from: (u32, u32),
    range_min: u32,
    range_max: u32,
    vertical: u32,
) -> Vec<(u32, u32)> {
    let height = crate::queries::terrain::height(terrain, from.0, from.1) as i32;
    let mut tiles = Vec::new();
    for row in 0..terrain.rows {
        for column in 0..terrain.columns {
            let distance = manhattan(from, (column, row));
            if distance < range_min || distance > range_max {
                continue;
            }
            if !crate::queries::terrain::is_open(terrain, column, row) {
                continue;
            }
            if (crate::queries::terrain::height(terrain, column, row) as i32 - height)
                .unsigned_abs()
                > vertical
            {
                continue;
            }
            tiles.push((column, row));
        }
    }
    tiles
}

pub fn area_tiles(terrain: &Terrain, center: (u32, u32), area: u32) -> Vec<(u32, u32)> {
    let height = crate::queries::terrain::height(terrain, center.0, center.1) as i32;
    let mut tiles = Vec::new();
    for row in 0..terrain.rows {
        for column in 0..terrain.columns {
            if manhattan(center, (column, row)) > area {
                continue;
            }
            if !crate::queries::terrain::is_open(terrain, column, row) {
                continue;
            }
            if (crate::queries::terrain::height(terrain, column, row) as i32 - height)
                .unsigned_abs()
                > 2
            {
                continue;
            }
            tiles.push((column, row));
        }
    }
    tiles
}

pub fn affected(
    roster: &[(Entity, Unit)],
    tiles: &[(u32, u32)],
    attacker: &Unit,
    ability: Ability,
) -> Vec<usize> {
    let factor = air_factor(attacker, ability);
    roster
        .iter()
        .enumerate()
        .filter(|(_, (_, unit))| tiles.contains(&(unit.column, unit.row)))
        .filter(|(_, (_, unit))| !ability.helps() || unit.team == attacker.team)
        .filter(|(_, (_, unit))| ability.helps() || !is_air(unit) || factor > 0.0)
        .map(|(index, _)| index)
        .collect()
}

pub fn tower_bonus(sites: &[(Entity, Structure)], team: Team, tile: (u32, u32)) -> i32 {
    let near = sites.iter().any(|(_, structure)| {
        structure.kind == StructureKind::Tower
            && structure.owner == Some(team)
            && manhattan((structure.column, structure.row), tile) <= theme::TOWER_RANGE
    });
    if near { theme::TOWER_HIT } else { 0 }
}

pub fn predict(
    terrain: &Terrain,
    sites: &[(Entity, Structure)],
    roster: &[(Entity, Unit)],
    attacker: &Unit,
    attacker_tile: (u32, u32),
    target: &Unit,
    ability: Ability,
) -> Strike {
    let height_gain = crate::queries::terrain::height(terrain, attacker_tile.0, attacker_tile.1)
        as i32
        - crate::queries::terrain::height(terrain, target.column, target.row) as i32;
    let side = side_of_attack(attacker_tile, (target.column, target.row), target.facing);
    let support = support_rank(roster, attacker, attacker_tile) as i32;
    strike(StrikeInput {
        attacker_attack: attacker.attack + support * theme::SUPPORT_POWER,
        attacker_magic: attacker.magic + support * theme::SUPPORT_POWER,
        attacker_suppressed: attacker.suppressed > 0,
        target_defense: target.defense,
        target_resist: target.resist,
        target_evasion: target.evasion,
        target_cover: if is_air(target) {
            crate::data::Feature::Open
        } else {
            crate::queries::terrain::feature(terrain, target.column, target.row)
        },
        target_air: is_air(target),
        height_gain: if is_air(attacker) || is_air(target) {
            0
        } else {
            height_gain
        },
        side,
        hit_bonus: tower_bonus(sites, attacker.team, attacker_tile) + support * theme::SUPPORT_HIT,
        air_factor: air_factor(attacker, ability),
        ability,
    })
}

pub fn can_counter(
    terrain: &Terrain,
    target: &Unit,
    attacker: &Unit,
    attacker_tile: (u32, u32),
    ability: Ability,
) -> bool {
    if ability.helps() || target.hp <= 0 || target.stunned > 0 {
        return false;
    }
    if ability.shape().effect != Effect::Physical {
        return false;
    }
    if side_of_attack(attacker_tile, (target.column, target.row), target.facing) == Side::Back {
        return false;
    }
    if is_air(attacker) && air_factor(target, Ability::Attack) <= 0.0 {
        return false;
    }
    let (range_min, range_max, vertical) = ability_range(target, Ability::Attack);
    let distance = manhattan((target.column, target.row), attacker_tile);
    let rise = (crate::queries::terrain::height(terrain, target.column, target.row) as i32
        - crate::queries::terrain::height(terrain, attacker_tile.0, attacker_tile.1) as i32)
        .unsigned_abs();
    distance >= range_min && distance <= range_max && rise <= vertical
}

pub fn ready_unit(roster: &[(Entity, Unit)]) -> Option<Entity> {
    roster
        .iter()
        .filter(|(_, unit)| unit.ct >= theme::READY_CT)
        .max_by_key(|(_, unit)| unit.ct)
        .map(|(entity, _)| *entity)
}

pub fn turn_order(roster: &[(Entity, Unit)], length: usize) -> Vec<Entity> {
    let mut clocks: Vec<(Entity, i32, i32)> = roster
        .iter()
        .map(|(entity, unit)| (*entity, unit.ct, unit.speed.max(1)))
        .collect();
    let mut order = Vec::new();
    let mut ticks = 0;
    while order.len() < length && !clocks.is_empty() && ticks < theme::TICK_LIMIT {
        let ready = clocks
            .iter_mut()
            .filter(|(_, ct, _)| *ct >= theme::READY_CT)
            .max_by_key(|(_, ct, _)| *ct);
        if let Some((entity, ct, _)) = ready {
            order.push(*entity);
            *ct = 0;
            continue;
        }
        for (_, ct, speed) in &mut clocks {
            *ct += *speed;
        }
        ticks += 1;
    }
    order
}

pub fn living(roster: &[(Entity, Unit)], team: Team) -> usize {
    roster.iter().filter(|(_, unit)| unit.team == team).count()
}

pub fn headquarters_held(sites: &[(Entity, Structure)], team: Team) -> bool {
    sites.iter().any(|(_, structure)| {
        structure.kind == StructureKind::Headquarters && structure.owner == Some(team)
    })
}

pub fn usable_abilities(unit: &Unit, moved: bool) -> Vec<Ability> {
    let mut abilities = vec![Ability::Attack];
    abilities.extend(unit.class.sheet().abilities.iter().copied());
    abilities
        .into_iter()
        .filter(|ability| ability.shape().mp <= unit.mp)
        .filter(|ability| !(moved && is_stationary(unit, *ability)))
        .collect()
}

pub fn can_capture(sites: &[(Entity, Structure)], unit: &Unit) -> Option<usize> {
    if !unit.class.sheet().can_capture {
        return None;
    }
    site_at(sites, unit.column, unit.row).filter(|index| sites[*index].1.owner != Some(unit.team))
}

fn capture_worth(kind: StructureKind, owner: Option<Team>, actor: Team) -> f32 {
    let base = match kind {
        StructureKind::Headquarters => 600.0,
        StructureKind::Depot => 60.0,
        StructureKind::Tower => 45.0,
        StructureKind::Factory => 30.0,
    };
    if owner.is_some_and(|owner| owner != actor) {
        base * 1.2
    } else {
        base
    }
}

pub fn plan_turn(
    terrain: &Terrain,
    roster: &[(Entity, Unit)],
    sites: &[(Entity, Structure)],
    actor_index: usize,
) -> Plan {
    let actor = &roster[actor_index].1;
    let reach = reachable(terrain, roster, actor);
    let candidates = destinations(terrain, &reach, roster, actor);
    let here = (actor.column, actor.row);
    let danger = danger_field(terrain, roster, actor.team);
    let hurt = 1.0 - hp_share(actor);
    let caution = theme::AI_CAUTION * (1.0 + hurt * theme::AI_HURT_CAUTION);
    let exposure = |column: u32, row: u32| -> f32 {
        if danger[crate::queries::terrain::index(terrain, column, row)] {
            caution
        } else {
            0.0
        }
    };
    let mut best: Option<(f32, Plan)> = None;
    let mut consider = |score: f32, plan: Plan| {
        if best
            .as_ref()
            .is_none_or(|(best_score, _)| score > *best_score)
        {
            best = Some((score, plan));
        }
    };

    for stand in &candidates {
        let steps = reach
            .iter()
            .find(|reach| (reach.column, reach.row) == *stand)
            .map_or(0, |reach| reach.steps);
        let moved = *stand != here;

        if actor.class.sheet().can_capture
            && let Some(index) = site_at(sites, stand.0, stand.1)
            && sites[index].1.owner != Some(actor.team)
        {
            let structure = &sites[index].1;
            let worth = capture_worth(structure.kind, structure.owner, actor.team);
            consider(
                worth - steps as f32 * 0.5 - exposure(stand.0, stand.1),
                Plan {
                    destination: *stand,
                    action: None,
                    capture: true,
                },
            );
        }

        for ability in usable_abilities(actor, moved) {
            let shape = ability.shape();
            let (range_min, range_max, vertical) = ability_range(actor, ability);
            for target in range_tiles(terrain, *stand, range_min, range_max, vertical) {
                let area = area_tiles(terrain, target, shape.area);
                let mut score = 0.0;
                let mut any = false;
                for (index, (_, unit)) in roster.iter().enumerate() {
                    let tile = if index == actor_index {
                        *stand
                    } else {
                        (unit.column, unit.row)
                    };
                    if !area.contains(&tile) {
                        continue;
                    }
                    if !ability.helps() && is_air(unit) && air_factor(actor, ability) <= 0.0 {
                        continue;
                    }
                    let strike = predict(terrain, sites, roster, actor, *stand, unit, ability);
                    match shape.effect {
                        Effect::Heal => {
                            if unit.team == actor.team {
                                let missing = (unit.max_hp - unit.hp).max(0);
                                if missing > 0 {
                                    any = true;
                                    score += strike.amount.min(missing) as f32 * 1.2;
                                }
                            }
                        }
                        Effect::Physical | Effect::Magic => {
                            let expected = strike.amount as f32 * strike.hit_chance as f32 / 100.0;
                            if unit.team != actor.team {
                                any = true;
                                score += expected;
                                if strike.amount >= unit.hp {
                                    score += 45.0;
                                }
                                if shape.inflicts.is_some() {
                                    score += 12.0;
                                }
                            } else if index == actor_index {
                                score -= expected * 2.5;
                            } else {
                                score -= expected * 1.5;
                            }
                        }
                    }
                }
                if !any || score <= 0.0 {
                    continue;
                }
                score -= steps as f32 * 0.5;
                score += crate::queries::terrain::height(terrain, stand.0, stand.1) as f32 * 0.8;
                score += crate::queries::terrain::feature(terrain, stand.0, stand.1).defense()
                    as f32
                    * 2.0;
                score -= exposure(stand.0, stand.1);
                score += theme::AI_FOCUS * helpers(roster, actor_index, target) as f32;
                consider(
                    score,
                    Plan {
                        destination: *stand,
                        action: Some((ability, target)),
                        capture: false,
                    },
                );
            }
        }
    }

    if let Some((_, plan)) = best {
        return plan;
    }

    let mut goals: Vec<(u32, u32)> = roster
        .iter()
        .filter(|(_, unit)| unit.team != actor.team)
        .map(|(_, unit)| (unit.column, unit.row))
        .collect();
    if actor.class.sheet().can_capture {
        goals.extend(
            sites
                .iter()
                .filter(|(_, structure)| structure.owner != Some(actor.team))
                .map(|(_, structure)| (structure.column, structure.row)),
        );
    }
    let mut destination = here;
    let mut best_score = f32::MIN;
    for stand in &candidates {
        let Some(distance) = goals
            .iter()
            .map(|goal| {
                manhattan(*stand, *goal)
                    + (crate::queries::terrain::height(terrain, stand.0, stand.1) as i32
                        - crate::queries::terrain::height(terrain, goal.0, goal.1) as i32)
                        .unsigned_abs()
                        / 2
            })
            .min()
        else {
            break;
        };
        let score = -(distance as f32) - exposure(stand.0, stand.1);
        if score > best_score {
            best_score = score;
            destination = *stand;
        }
    }
    Plan {
        destination,
        action: None,
        capture: false,
    }
}

pub fn danger_field(terrain: &Terrain, roster: &[(Entity, Unit)], team: Team) -> Vec<bool> {
    let mut threatened = vec![false; terrain.heights.len()];
    for (_, foe) in roster.iter().filter(|(_, unit)| unit.team != team) {
        let reach = reachable(terrain, roster, foe);
        let stands = destinations(terrain, &reach, roster, foe);
        let (range_min, range_max, vertical) = ability_range(foe, Ability::Attack);
        for stand in stands {
            for (column, row) in range_tiles(terrain, stand, range_min, range_max, vertical) {
                threatened[crate::queries::terrain::index(terrain, column, row)] = true;
            }
        }
    }
    threatened
}

pub fn danger_tiles(terrain: &Terrain, roster: &[(Entity, Unit)], team: Team) -> Vec<(u32, u32)> {
    let threatened = danger_field(terrain, roster, team);
    (0..terrain.rows)
        .flat_map(|row| (0..terrain.columns).map(move |column| (column, row)))
        .filter(|(column, row)| threatened[crate::queries::terrain::index(terrain, *column, *row)])
        .collect()
}

fn stick_step(value: f32) -> i32 {
    if value > theme::STICK_STEP {
        1
    } else if value < -theme::STICK_STEP {
        -1
    } else {
        0
    }
}

pub fn direction(actions: &Actions<Ask>) -> (i32, i32) {
    let reach = actions.axis2(Ask::Move);
    let column = stick_step(reach.x) + i32::from(actions.pressed(Ask::Right))
        - i32::from(actions.pressed(Ask::Left));
    let row = stick_step(reach.y) + i32::from(actions.pressed(Ask::Up))
        - i32::from(actions.pressed(Ask::Down));
    (column.clamp(-1, 1), row.clamp(-1, 1))
}

pub fn menu_step(actions: &Actions<Ask>) -> i32 {
    if actions.just_pressed(Ask::MenuUp) {
        return 1;
    }
    if actions.just_pressed(Ask::MenuDown) {
        return -1;
    }
    if actions.just_pressed(Ask::Move) {
        return stick_step(actions.axis2(Ask::Move).y);
    }
    0
}

pub fn tile_under_ray(terrain: &Terrain, ray: &PickingRay) -> Option<(u32, u32)> {
    let top = theme::MAX_HEIGHT as f32 + 0.5;
    let mut distance = 0.0;
    if ray.origin.y > top && ray.direction.y < 0.0 {
        distance = (top - ray.origin.y) / ray.direction.y;
    }
    while distance < theme::RAY_REACH {
        let point = ray.origin + ray.direction * distance;
        if point.y < 0.0 {
            return None;
        }
        if let Some((column, row)) = crate::queries::terrain::column_of(terrain, point.x, point.z)
            && point.y <= crate::queries::terrain::stand(terrain, column, row).y
        {
            return Some((column, row));
        }
        distance += theme::RAY_STEP;
    }
    None
}

pub fn nearest_tile(
    terrain: &Terrain,
    wanted: (u32, u32),
    accept: impl Fn(u32, u32) -> bool,
) -> (u32, u32) {
    let mut best = wanted;
    let mut best_distance = u32::MAX;
    for row in 0..terrain.rows {
        for column in 0..terrain.columns {
            if !accept(column, row) {
                continue;
            }
            let distance = manhattan(wanted, (column, row));
            if distance < best_distance {
                best_distance = distance;
                best = (column, row);
            }
        }
    }
    best
}

pub fn digit_offset(slot: usize, count: usize) -> nalgebra_glm::Vec3 {
    let width = theme::DIGIT_SIZE * 0.8;
    nalgebra_glm::Vec3::new((slot as f32 - (count as f32 - 1.0) / 2.0) * width, 0.0, 0.0)
}

pub fn hp_share(unit: &Unit) -> f32 {
    if unit.max_hp <= 0 {
        0.0
    } else {
        (unit.hp as f32 / unit.max_hp as f32).clamp(0.0, 1.0)
    }
}

pub fn mp_share(unit: &Unit) -> f32 {
    if unit.max_mp <= 0 {
        0.0
    } else {
        (unit.mp as f32 / unit.max_mp as f32).clamp(0.0, 1.0)
    }
}

pub fn status_text(unit: &Unit) -> String {
    let mut parts = Vec::new();
    if unit.stunned > 0 {
        parts.push(crate::data::Status::Stunned.name());
    }
    if unit.suppressed > 0 {
        parts.push(crate::data::Status::Suppressed.name());
    }
    parts.join("  ")
}

pub fn opening_screen(arguments: &SrpgArguments) -> Screen {
    if arguments.battle {
        Screen::Battle
    } else if arguments.camp {
        Screen::Camp
    } else if arguments.prologue {
        Screen::Prologue
    } else {
        Screen::Title
    }
}

pub fn opening_chapter(arguments: &SrpgArguments) -> usize {
    arguments.chapter.unwrap_or(1).saturating_sub(1)
}

fn helpers(roster: &[(Entity, Unit)], actor_index: usize, target: (u32, u32)) -> usize {
    roster
        .iter()
        .enumerate()
        .filter(|(index, (_, unit))| {
            *index != actor_index && unit.team == roster[actor_index].1.team && unit.hp > 0 && {
                let (_, range_max, _) = ability_range(unit, Ability::Attack);
                manhattan((unit.column, unit.row), target) <= effective_move(unit) + range_max
            }
        })
        .count()
}
