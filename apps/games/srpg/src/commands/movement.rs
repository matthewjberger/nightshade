use crate::commands::{edit_structure, edit_unit, log, set_phase};
use crate::data::Team;
use crate::queries::site_at;
use crate::queries::{Structures, Units};
use crate::resources::{Battle, Cursor, Phase, Terrain};
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_scene::prelude::*;

pub fn finish_walk(
    battle: &mut Battle,
    cursor: &mut Cursor,
    terrain: &Terrain,
    units: &mut Units<'_, '_>,
    transforms: &mut Query<'_, (&mut LocalTransform,)>,
    structures: &mut Structures<'_, '_>,
    active: Entity,
) {
    let destination = battle.path.last().copied();
    let mut team = Team::Player;
    let mut name = "";
    let mut tile = (0, 0);
    let mut can_capture = false;
    let mut stand = None;
    edit_unit(units, active, |unit| {
        if let Some((column, row)) = destination {
            unit.column = column;
            unit.row = row;
        }
        stand = Some(
            crate::queries::terrain::stand(terrain, unit.column, unit.row)
                + Vec3::new(0.0, unit.mobility.hover(), 0.0),
        );
        team = unit.team;
        name = unit.name;
        tile = (unit.column, unit.row);
        can_capture = unit.class.sheet().can_capture;
        cursor.column = unit.column;
        cursor.row = unit.row;
    });
    if let Some(at) = stand {
        transforms.for_entity(active, |(transform,)| transform.translation = at);
    }
    battle.path.clear();
    let planned = battle.plan.is_some();
    match team {
        Team::Player if !planned => {
            battle.menu_index = 0;
            set_phase(battle, Phase::Menu);
        }
        _ => {
            let plan = battle.plan;
            if plan.is_some_and(|plan| plan.capture) && can_capture {
                let sites = crate::queries::sites(structures);
                if let Some(index) = site_at(&sites, tile.0, tile.1) {
                    let entity = sites[index].0;
                    let kind = sites[index].1.kind;
                    edit_structure(structures, entity, |structure| {
                        crate::commands::capture(structure, team)
                    });
                    log(battle, format!("{} captures the {}.", name, kind.name()));
                    battle.acted = true;
                }
                set_phase(battle, Phase::TurnEnd);
                return;
            }
            match plan.and_then(|plan| plan.action) {
                Some((ability, target)) => {
                    battle.ability = Some(ability);
                    battle.target_tile = Some(target);
                    battle.acted = true;
                    set_phase(battle, Phase::Acting);
                }
                None => set_phase(battle, Phase::TurnEnd),
            }
        }
    }
}
