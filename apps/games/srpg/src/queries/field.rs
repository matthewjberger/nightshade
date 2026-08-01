pub fn ground_of(plan: &Plan, column: u32, row: u32) -> u8 {
    plan.ground
        .get(row as usize)
        .and_then(|line| line.as_bytes().get(column as usize))
        .copied()
        .unwrap_or(b'3')
}

pub fn mark_of(plan: &Plan, column: u32, row: u32) -> u8 {
    plan.props
        .get(row as usize)
        .and_then(|line| line.as_bytes().get(column as usize))
        .copied()
        .unwrap_or(b'.')
}

pub fn camp_is_road(terrain: &Terrain, column: u32, row: u32) -> bool {
    let (low, high) = theme::CAMP_PLAZA;
    let plaza = (low..=high).contains(&column) && (low..=high).contains(&row);
    let lane = column == 9 || row == 9;
    !crate::queries::terrain::is_water(terrain, column, row) && (plaza || lane)
}

pub fn is_road(terrain: &Terrain, column: u32, row: u32) -> bool {
    terrain
        .roads
        .get(crate::queries::terrain::index(terrain, column, row))
        .copied()
        .unwrap_or(false)
        && !crate::queries::terrain::is_water(terrain, column, row)
}

pub fn is_bank(terrain: &Terrain, column: u32, row: u32) -> bool {
    if crate::queries::terrain::is_water(terrain, column, row) {
        return false;
    }
    if crate::queries::terrain::height(terrain, column, row) > 2 {
        return false;
    }
    neighbors(terrain, column, row)
        .into_iter()
        .flatten()
        .any(|(near_column, near_row)| {
            crate::queries::terrain::is_water(terrain, near_column, near_row)
        })
}

use crate::data::{Mobility, Plan};
use crate::resources::Terrain;
use crate::theme;
use nalgebra_glm::Vec2;

pub fn near_spawn(plan: &Plan, column: u32, row: u32) -> bool {
    spawn_tiles(plan).any(|(spawn_column, spawn_row, _)| {
        spawn_column.abs_diff(column) <= 1 && spawn_row.abs_diff(row) <= 1
    }) || is_site(plan, column, row)
}

pub fn spawn_tiles(plan: &Plan) -> impl Iterator<Item = (u32, u32, Mobility)> + '_ {
    let enemy = plan
        .squad
        .iter()
        .map(|recruit| (recruit.column, recruit.row, recruit.class.sheet().mobility));
    let ours = plan
        .spots
        .iter()
        .map(|(column, row)| (*column, *row, Mobility::Foot));
    enemy.chain(ours)
}

pub fn is_land_spawn(plan: &Plan, column: u32, row: u32) -> bool {
    spawn_tiles(plan)
        .filter(|(_, _, mobility)| *mobility != Mobility::Naval)
        .any(|(spawn_column, spawn_row, _)| spawn_column == column && spawn_row == row)
}

pub fn is_water_spawn(plan: &Plan, column: u32, row: u32) -> bool {
    spawn_tiles(plan)
        .filter(|(_, _, mobility)| *mobility == Mobility::Naval)
        .any(|(spawn_column, spawn_row, _)| spawn_column == column && spawn_row == row)
}

pub fn is_site(plan: &Plan, column: u32, row: u32) -> bool {
    plan.sites
        .iter()
        .any(|site| site.column == column && site.row == row)
}

pub fn neighbors(terrain: &Terrain, column: u32, row: u32) -> [Option<(u32, u32)>; 4] {
    let left = (column > 0).then(|| (column - 1, row));
    let right = (column + 1 < terrain.columns).then(|| (column + 1, row));
    let back = (row > 0).then(|| (column, row - 1));
    let front = (row + 1 < terrain.rows).then(|| (column, row + 1));
    [left, right, back, front]
}

pub fn walkable(terrain: &Terrain, ground: Vec2) -> bool {
    let Some((column, row)) = crate::queries::terrain::column_of(terrain, ground.x, ground.y)
    else {
        return false;
    };
    crate::queries::terrain::can_stop(terrain, column, row, crate::data::Mobility::Foot)
}
