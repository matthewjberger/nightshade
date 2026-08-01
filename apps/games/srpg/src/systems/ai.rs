use crate::commands::{edit_structure, log, look_at, set_phase};
use crate::queries::{
    Structures, Units, path_to, plan_turn, reachable, roster, roster_index, site_at, sites,
};
use crate::resources::{Battle, CameraRig, Phase, Terrain};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub fn think(
    time: Res<Time>,
    terrain: Res<Terrain>,
    mut battle: ResMut<Battle>,
    mut rig: ResMut<CameraRig>,
    mut units: Units<'_, '_>,
    mut structures: Structures<'_, '_>,
) {
    if battle.phase != Phase::Thinking {
        return;
    }
    let Some(active) = battle.active else {
        set_phase(&mut battle, Phase::TurnEnd);
        return;
    };
    let roster = roster(&mut units);
    let Some(index) = roster_index(&roster, active) else {
        set_phase(&mut battle, Phase::TurnEnd);
        return;
    };
    let actor = roster[index].1.clone();
    let sites = sites(&mut structures);

    if battle.plan.is_none() {
        let plan = plan_turn(&terrain, &roster, &sites, index);
        battle.plan = Some(plan);
        look_at(
            &mut rig,
            crate::queries::terrain::stand(&terrain, actor.column, actor.row),
        );
        let words = if plan.capture {
            format!("{} moves to seize ground.", actor.name)
        } else {
            match plan.action {
                Some((ability, _)) => format!("{} readies {}.", actor.name, ability.name()),
                None if plan.destination != (actor.column, actor.row) => {
                    format!("{} advances.", actor.name)
                }
                None => format!("{} holds position.", actor.name),
            }
        };
        log(&mut battle, words);
    }

    battle.timer += time.delta_time;
    if battle.timer < theme::THINK_PAUSE {
        return;
    }
    let Some(plan) = battle.plan else {
        set_phase(&mut battle, Phase::TurnEnd);
        return;
    };
    if plan.destination != (actor.column, actor.row) {
        let reach = reachable(&terrain, &roster, &actor);
        let path = path_to(&reach, plan.destination);
        if path.len() >= 2 {
            battle.path = path;
            battle.walk_progress = 0.0;
            battle.moved = true;
            set_phase(&mut battle, Phase::Walking);
            return;
        }
    }
    if plan.capture {
        if let Some(site_index) = site_at(&sites, actor.column, actor.row) {
            let entity = sites[site_index].0;
            let kind = sites[site_index].1.kind;
            edit_structure(&mut structures, entity, |structure| {
                crate::commands::capture(structure, actor.team)
            });
            log(
                &mut battle,
                format!("{} captures the {}.", actor.name, kind.name()),
            );
            battle.acted = true;
        }
        set_phase(&mut battle, Phase::TurnEnd);
        return;
    }
    match plan.action {
        Some((ability, tile)) => {
            battle.ability = Some(ability);
            battle.target_tile = Some(tile);
            battle.acted = true;
            set_phase(&mut battle, Phase::Acting);
        }
        None => set_phase(&mut battle, Phase::TurnEnd),
    }
}
