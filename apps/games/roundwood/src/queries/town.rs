use crate::data::{self, Ground};
use crate::queries::cylinder;
use crate::resources::Town;
use nalgebra_glm::Vec2;

pub fn index(town: &Town, column: u32, row: u32) -> usize {
    (row * town.columns + column) as usize
}

pub fn cell_of(town: &Town, spot: Vec2) -> Option<(u32, u32)> {
    if spot.x < 0.0 || spot.x >= town.columns as f32 {
        return None;
    }
    let row = cylinder::wrap(spot.y).floor() as u32 % town.rows;
    Some((spot.x.floor() as u32, row))
}

pub fn height(town: &Town, column: u32, row: u32) -> u32 {
    town.heights[index(town, column, row)]
}

pub fn ground(town: &Town, column: u32, row: u32) -> Ground {
    town.ground[index(town, column, row)]
}

pub fn blocked(town: &Town, column: u32, row: u32) -> bool {
    town.blocked[index(town, column, row)]
}

pub fn walkable(town: &Town, column: u32, row: u32) -> bool {
    ground(town, column, row).walkable() && !blocked(town, column, row)
}

pub fn center(column: u32, row: u32) -> Vec2 {
    Vec2::new(column as f32 + 0.5, row as f32 + 0.5)
}

pub fn height_at(town: &Town, spot: Vec2) -> Option<u32> {
    cell_of(town, spot).map(|(column, row)| height(town, column, row))
}

pub fn lift_at(town: &Town, spot: Vec2) -> f32 {
    height_at(town, spot).unwrap_or(data::LAND_HEIGHT) as f32
}

pub fn can_step(town: &Town, from: Vec2, to: Vec2) -> bool {
    let (Some(here), Some(there)) = (cell_of(town, from), cell_of(town, to)) else {
        return false;
    };
    if !walkable(town, there.0, there.1) {
        return false;
    }
    let rise = height(town, there.0, there.1) as i32 - height(town, here.0, here.1) as i32;
    rise.abs() <= 1
}

pub fn nearest_walkable(town: &Town, spot: Vec2) -> Vec2 {
    if let Some((column, row)) = cell_of(town, spot)
        && walkable(town, column, row)
    {
        return spot;
    }
    let mut best: Option<(f32, Vec2)> = None;
    for row in 0..town.rows {
        for column in 0..town.columns {
            if !walkable(town, column, row) {
                continue;
            }
            let middle = center(column, row);
            let distance = cylinder::town_distance(spot, middle);
            if best.is_none_or(|(near, _)| distance < near) {
                best = Some((distance, middle));
            }
        }
    }
    best.map_or(spot, |(_, middle)| middle)
}

pub fn open(town: &Town, column: u32, row: u32) -> bool {
    walkable(town, column, row) && ground(town, column, row) == Ground::Grass
}

pub fn neighbors(town: &Town, column: u32, row: u32) -> [Option<(u32, u32)>; 4] {
    let west = (column > 0).then(|| (column - 1, row));
    let east = (column + 1 < town.columns).then(|| (column + 1, row));
    let back = Some((column, (row + town.rows - 1) % town.rows));
    let front = Some((column, (row + 1) % town.rows));
    [west, east, back, front]
}

pub fn on_road(column: u32) -> bool {
    (data::ROAD_START..data::ROAD_END).contains(&column)
}

pub fn in_dock(column: u32, row: u32) -> bool {
    (data::DOCK_COLUMN_START..data::DOCK_COLUMN_END).contains(&column)
        && (data::DOCK_ROW_START..data::DOCK_ROW_END).contains(&row)
}

pub fn in_pond(column: u32, row: u32) -> bool {
    let across = column as f32 + 0.5 - data::POND_COLUMN;
    let along = cylinder::ring_delta(row as f32 + 0.5, data::POND_ROW);
    (across * across + along * along).sqrt() < data::POND_RADIUS
}

pub fn on_pond_deck(column: u32, row: u32) -> bool {
    (data::POND_DECK_COLUMN_START..data::POND_DECK_COLUMN_END).contains(&column)
        && row == data::POND_DECK_ROW
}

pub fn hill_rise(column: u32, row: u32) -> u32 {
    let across = column as f32 + 0.5 - data::HILL_COLUMN;
    let along = cylinder::ring_delta(row as f32 + 0.5, data::HILL_ROW);
    let distance = (across * across + along * along).sqrt();
    ((data::HILL_RADIUS - distance) / data::HILL_STEP)
        .floor()
        .clamp(0.0, data::HILL_RISE) as u32
}

pub fn shaped(column: u32, row: u32) -> (u32, Ground) {
    if in_dock(column, row) || on_pond_deck(column, row) {
        return (data::LAND_HEIGHT, Ground::Plank);
    }
    if (data::RIVER_START..data::RIVER_END).contains(&row) {
        if on_road(column) {
            return (data::LAND_HEIGHT, Ground::Plank);
        }
        return (data::WATER_HEIGHT, Ground::Water);
    }
    if !(data::WEST_SEA_END..data::EAST_SEA_START).contains(&column) {
        return (data::WATER_HEIGHT, Ground::Water);
    }
    if !(data::WEST_BEACH_END..data::EAST_BEACH_START).contains(&column) {
        return (data::LAND_HEIGHT, Ground::Sand);
    }
    if in_pond(column, row) {
        return (data::WATER_HEIGHT, Ground::Water);
    }
    if on_road(column) {
        return (data::LAND_HEIGHT, Ground::Path);
    }
    let in_plaza = (data::PLAZA_COLUMN_START..data::PLAZA_COLUMN_END).contains(&column)
        && (data::PLAZA_ROW_START..data::PLAZA_ROW_END).contains(&row);
    if in_plaza {
        return (data::LAND_HEIGHT, Ground::Path);
    }
    (data::LAND_HEIGHT + hill_rise(column, row), Ground::Grass)
}

pub fn place_name(town: &Town, spot: Vec2) -> &'static str {
    if let Some((column, row)) = cell_of(town, spot) {
        if in_dock(column, row) {
            return data::THE_DOCK;
        }
        if column < data::WEST_BEACH_END {
            return data::WEST_STRAND;
        }
        if column >= data::EAST_BEACH_START {
            return data::EAST_STRAND;
        }
    }
    let mut best = data::PLACES[0];
    for place in data::PLACES {
        if cylinder::ring_delta(spot.y, place.0).abs() < cylinder::ring_delta(spot.y, best.0).abs()
        {
            best = place;
        }
    }
    best.1
}
