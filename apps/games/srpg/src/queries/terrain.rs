use crate::data::{Feature, Mobility};
use crate::queries::field::{is_bank, is_road};
use crate::resources::Terrain;
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};

pub fn column_of(terrain: &Terrain, x: f32, z: f32) -> Option<(u32, u32)> {
    let column = (x - terrain.origin.x).floor();
    let row = (z - terrain.origin.y).floor();
    if column < 0.0 || row < 0.0 || column >= terrain.columns as f32 || row >= terrain.rows as f32 {
        return None;
    }
    Some((column as u32, row as u32))
}

pub fn contains(terrain: &Terrain, column: i32, row: i32) -> bool {
    column >= 0 && row >= 0 && column < terrain.columns as i32 && row < terrain.rows as i32
}

pub fn index(terrain: &Terrain, column: u32, row: u32) -> usize {
    (row * terrain.columns + column) as usize
}

pub fn height(terrain: &Terrain, column: u32, row: u32) -> u32 {
    terrain
        .heights
        .get(index(terrain, column, row))
        .copied()
        .unwrap_or(theme::MIN_HEIGHT)
}

pub fn height_at(terrain: &Terrain, x: f32, z: f32) -> Option<u32> {
    column_of(terrain, x, z).map(|(column, row)| height(terrain, column, row))
}

pub fn feature(terrain: &Terrain, column: u32, row: u32) -> Feature {
    terrain
        .features
        .get(index(terrain, column, row))
        .copied()
        .unwrap_or(Feature::Open)
}

pub fn is_water(terrain: &Terrain, column: u32, row: u32) -> bool {
    height(terrain, column, row) <= theme::MIN_HEIGHT
}

pub fn is_land(terrain: &Terrain, column: u32, row: u32) -> bool {
    !is_water(terrain, column, row) && feature(terrain, column, row) != Feature::Obstacle
}

pub fn is_open(terrain: &Terrain, column: u32, row: u32) -> bool {
    feature(terrain, column, row) != Feature::Obstacle
}

pub fn passable(terrain: &Terrain, column: u32, row: u32, mobility: Mobility) -> bool {
    match mobility {
        Mobility::Air => true,
        Mobility::Naval => is_water(terrain, column, row),
        Mobility::Foot | Mobility::Walker => is_land(terrain, column, row),
        Mobility::Wheels | Mobility::Tracks => {
            is_land(terrain, column, row) && feature(terrain, column, row) != Feature::Tree
        }
    }
}

pub fn can_stop(terrain: &Terrain, column: u32, row: u32, mobility: Mobility) -> bool {
    passable(terrain, column, row, mobility) && is_open(terrain, column, row)
}

pub fn center(terrain: &Terrain, column: u32, row: u32) -> Vec2 {
    terrain.origin + Vec2::new(column as f32 + 0.5, row as f32 + 0.5)
}

pub fn top(terrain: &Terrain, column: u32, row: u32) -> Vec3 {
    let center = center(terrain, column, row);
    Vec3::new(center.x, height(terrain, column, row) as f32, center.y)
}

pub fn stand(terrain: &Terrain, column: u32, row: u32) -> Vec3 {
    let mut top = top(terrain, column, row);
    if is_water(terrain, column, row) {
        top.y = theme::WATER_LEVEL;
    }
    top
}

pub fn terrain_name(terrain: &Terrain, column: u32, row: u32) -> String {
    let ground = if crate::queries::terrain::is_water(terrain, column, row) {
        "Water"
    } else if is_road(terrain, column, row) {
        "Road"
    } else if is_bank(terrain, column, row) {
        "Sand"
    } else {
        "Grass"
    };
    let feature = crate::queries::terrain::feature(terrain, column, row);
    if feature == Feature::Open {
        ground.to_string()
    } else {
        format!("{}  {}", ground, feature.name())
    }
}
