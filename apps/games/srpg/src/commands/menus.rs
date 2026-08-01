use crate::commands::{
    begin_the_run, edit_fighters, edit_unit, enlist_again, look_at, set_phase, show_targets,
};
use crate::data::Unit;
use crate::data::{Ability, Team};
use crate::queries::{Fighters, Roster, Units, ability_range, manhattan, range_tiles};
use crate::resources::{
    Battle, CameraRig, Campaign, Cursor, Highlights, Pause, Phase, Screen, Terrain,
};
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_state::prelude::*;

pub fn hold_the_game(pause: &mut Pause, next: &mut NextState<Screen>, from: Screen) {
    pause.from = from;
    pause.index = 0;
    next.pending = Some(Screen::Paused);
}

pub fn start_the_run(
    campaign: &mut Campaign,
    next: &mut NextState<Screen>,
    fighters: &mut Fighters<'_, '_>,
) {
    edit_fighters(fighters, enlist_again);
    begin_the_run(campaign);
    next.pending = Some(Screen::Prologue);
}

pub fn aim(
    terrain: &Terrain,
    highlights: &mut Highlights,
    cursor: &mut Cursor,
    rig: &mut CameraRig,
    battle: &mut Battle,
    roster: &Roster,
    unit: &Unit,
    ability: Ability,
) {
    let (range_min, range_max, vertical) = ability_range(unit, ability);
    let here = (unit.column, unit.row);
    let tiles = range_tiles(terrain, here, range_min, range_max, vertical);
    let wanted_team = if ability.helps() {
        unit.team
    } else {
        match unit.team {
            Team::Player => Team::Enemy,
            Team::Enemy => Team::Player,
        }
    };
    let first = roster
        .iter()
        .filter(|(_, other)| {
            other.team == wanted_team && tiles.contains(&(other.column, other.row))
        })
        .min_by_key(|(_, other)| manhattan(here, (other.column, other.row)))
        .map(|(_, other)| (other.column, other.row))
        .or_else(|| tiles.first().copied())
        .unwrap_or(here);
    let stand = crate::queries::terrain::stand(terrain, first.0, first.1);
    show_targets(highlights, tiles.clone());
    cursor.column = first.0;
    cursor.row = first.1;
    look_at(rig, stand);
    battle.ability = Some(ability);
    battle.targets = tiles;
    set_phase(battle, Phase::ChooseTarget);
}

pub fn take_back_the_move(
    battle: &mut Battle,
    terrain: &Terrain,
    cursor: &mut Cursor,
    rig: &mut CameraRig,
    units: &mut Units<'_, '_>,
    transforms: &mut Query<'_, (&mut LocalTransform,)>,
    active: Entity,
) {
    let (column, row, facing) = (battle.start_column, battle.start_row, battle.start_facing);
    let stand = crate::queries::terrain::stand(terrain, column, row);
    let mut name = "";
    let mut hover = 0.0;
    edit_unit(units, active, |unit| {
        unit.column = column;
        unit.row = row;
        unit.facing = facing;
        hover = unit.mobility.hover();
        name = unit.name;
    });
    transforms.for_entity(active, |(transform,)| {
        transform.translation = stand + Vec3::new(0.0, hover, 0.0)
    });
    cursor.column = column;
    cursor.row = row;
    look_at(rig, stand);
    battle.moved = false;
    battle.path.clear();
    crate::commands::log(battle, format!("{name} steps back."));
    set_phase(battle, Phase::Menu);
}
