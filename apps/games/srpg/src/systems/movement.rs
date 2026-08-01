use crate::commands::movement::finish_walk;
use crate::commands::{clear_highlights, edit_unit, look_at, set_phase, show_path};
use crate::queries::{
    Structures, Units, destinations, facing_toward, path_to, roster, roster_index,
};
use crate::resources::{Battle, CameraRig, Cursor, Highlights, Phase, Terrain};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn choose_the_move(
    mut battle: ResMut<Battle>,
    mut cursor: ResMut<Cursor>,
    terrain: Res<Terrain>,
    mut highlights: ResMut<Highlights>,
    mut camera_rig: ResMut<CameraRig>,
    mut units: Units<'_, '_>,
) {
    if battle.phase != Phase::ChooseMove {
        return;
    }
    let Some(active) = battle.active else {
        return;
    };
    let roster = roster(&mut units);
    let Some(index) = roster_index(&roster, active) else {
        return;
    };
    let unit = roster[index].1.clone();
    let (cursor_tile, confirm, cancel) =
        { ((cursor.column, cursor.row), cursor.confirm, cursor.cancel) };
    let here = (unit.column, unit.row);
    let stand = crate::queries::terrain::stand(&terrain, unit.column, unit.row);

    let (reachable_tiles, path) = {
        let tiles = destinations(&terrain, &battle.reachable, &roster, &unit);
        let path = if tiles.contains(&cursor_tile) && cursor_tile != here {
            path_to(&battle.reachable, cursor_tile)
        } else {
            Vec::new()
        };
        (tiles, path)
    };
    show_path(&mut highlights, path.clone());

    if cancel || (confirm && cursor_tile == here) {
        clear_highlights(&mut highlights);
        cursor.column = here.0;
        cursor.row = here.1;
        look_at(&mut camera_rig, stand);
        set_phase(&mut battle, Phase::Menu);
        return;
    }
    if confirm && reachable_tiles.contains(&cursor_tile) && path.len() >= 2 {
        clear_highlights(&mut highlights);
        battle.path = path;
        battle.walk_progress = 0.0;
        battle.moved = true;
        set_phase(&mut battle, Phase::Walking);
    }
}

pub fn walk_the_path(
    time: Res<Time>,
    terrain: Res<Terrain>,
    mut battle: ResMut<Battle>,
    mut rig: ResMut<CameraRig>,
    mut cursor: ResMut<Cursor>,
    mut units: Units<'_, '_>,
    mut transforms: Query<(&mut LocalTransform,)>,
    mut structures: Structures<'_, '_>,
) {
    if battle.phase != Phase::Walking {
        return;
    }
    let Some(active) = battle.active else {
        set_phase(&mut battle, Phase::TurnEnd);
        return;
    };
    if battle.path.len() < 2 {
        finish_walk(
            &mut battle,
            &mut cursor,
            &terrain,
            &mut units,
            &mut transforms,
            &mut structures,
            active,
        );
        return;
    }
    battle.walk_progress += time.delta_time * theme::WALK_SPEED;
    let last = battle.path.len() - 1;
    let segment = battle.walk_progress.floor() as usize;
    if segment >= last {
        finish_walk(
            &mut battle,
            &mut cursor,
            &terrain,
            &mut units,
            &mut transforms,
            &mut structures,
            active,
        );
        return;
    }
    let from = battle.path[segment];
    let to = battle.path[segment + 1];
    let share = battle.walk_progress - segment as f32;
    let start = crate::queries::terrain::stand(&terrain, from.0, from.1);
    let end = crate::queries::terrain::stand(&terrain, to.0, to.1);
    let facing = facing_toward(from, to);
    let mut placement = None;
    edit_unit(&mut units, active, |unit| {
        let hop = if unit.mobility.hover() > 0.0 {
            0.0
        } else {
            (share * std::f32::consts::PI).sin() * theme::HOP_HEIGHT
        };
        let position =
            start + (end - start) * share + Vec3::new(0.0, hop + unit.mobility.hover(), 0.0);
        placement = Some(position);
        if let Some(facing) = facing {
            unit.facing = facing;
        }
        let (column, row) = if share < 0.5 { from } else { to };
        unit.column = column;
        unit.row = row;
        look_at(&mut rig, position);
    });
    if let Some(position) = placement {
        transforms.for_entity(active, |(transform,)| transform.translation = position);
    }
}
