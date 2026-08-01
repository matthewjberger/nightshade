use crate::data::{HouseKind, Zone};
use crate::resources::{Farmland, Terrain};
use crate::theme;
use nalgebra_glm::Vec2;

pub fn column_of(terrain: &Terrain, x: f32, z: f32) -> Option<(u32, u32)> {
    let column = (x - terrain.origin.x).floor();
    let row = (z - terrain.origin.y).floor();
    if column < 0.0 || row < 0.0 || column >= terrain.columns as f32 || row >= terrain.rows as f32 {
        return None;
    }
    Some((column as u32, row as u32))
}

pub fn index(terrain: &Terrain, column: u32, row: u32) -> usize {
    (row * terrain.columns + column) as usize
}

pub fn height(terrain: &Terrain, column: u32, row: u32) -> u32 {
    terrain.heights[index(terrain, column, row)]
}

pub fn height_at(terrain: &Terrain, x: f32, z: f32) -> Option<u32> {
    column_of(terrain, x, z).map(|(column, row)| height(terrain, column, row))
}

pub fn center(terrain: &Terrain, column: u32, row: u32) -> Vec2 {
    terrain.origin + Vec2::new(column as f32 + 0.5, row as f32 + 0.5)
}

pub fn is_walkable(terrain: &Terrain, column: u32, row: u32) -> bool {
    terrain.walkable[index(terrain, column, row)]
}

pub fn is_occupied(terrain: &Terrain, column: u32, row: u32) -> bool {
    terrain.occupied[index(terrain, column, row)]
}

pub fn is_bridge(terrain: &Terrain, column: u32, row: u32) -> bool {
    terrain.bridge[index(terrain, column, row)]
}

pub fn is_water(terrain: &Terrain, column: u32, row: u32) -> bool {
    height(terrain, column, row) <= theme::MIN_HEIGHT
}

pub fn can_step(terrain: &Terrain, from: Vec2, to: Vec2) -> bool {
    let (Some(here), Some(there)) = (
        column_of(terrain, from.x, from.y),
        column_of(terrain, to.x, to.y),
    ) else {
        return false;
    };
    if !is_walkable(terrain, there.0, there.1) {
        return false;
    }
    let rise = height(terrain, there.0, there.1) as i32 - height(terrain, here.0, here.1) as i32;
    rise.abs() <= 1
}

pub fn neighbors(terrain: &Terrain, column: u32, row: u32) -> [Option<(u32, u32)>; 4] {
    let left = (column > 0).then(|| (column - 1, row));
    let right = (column + 1 < terrain.columns).then(|| (column + 1, row));
    let back = (row > 0).then(|| (column, row - 1));
    let front = (row + 1 < terrain.rows).then(|| (column, row + 1));
    [left, right, back, front]
}

pub fn beside_water(terrain: &Terrain, column: u32, row: u32) -> bool {
    neighbors(terrain, column, row)
        .into_iter()
        .flatten()
        .any(|(c, r)| is_water(terrain, c, r) && !is_bridge(terrain, c, r))
}

pub fn nearest_walkable(terrain: &Terrain, spot: Vec2) -> Vec2 {
    let mut best: Option<(f32, Vec2)> = None;
    for row in 0..terrain.rows {
        for column in 0..terrain.columns {
            if !is_walkable(terrain, column, row) || is_occupied(terrain, column, row) {
                continue;
            }
            let middle = center(terrain, column, row);
            let distance = (middle - spot).magnitude();
            if best.is_none_or(|(nearest, _)| distance < nearest) {
                best = Some((distance, middle));
            }
        }
    }
    best.map_or(spot, |(_, middle)| middle)
}

pub fn river_center(x: f32) -> f32 {
    4.0 + 3.0 * (0.2 * x).sin()
}

pub fn path_center(z: f32) -> f32 {
    -2.0 + 4.0 * (0.17 * z + 0.6).sin()
}

pub fn in_region(point: Vec2, min: Vec2, max: Vec2) -> bool {
    point.x >= min.x && point.x < max.x && point.y >= min.y && point.y < max.y
}

pub fn corner_of(column: u32, row: u32) -> Vec2 {
    theme::TERRAIN_ORIGIN + Vec2::new(column as f32, row as f32)
}

pub fn in_plot_plateau(column: u32, row: u32) -> bool {
    in_region(
        corner_of(column, row),
        theme::PLOT_ORIGIN - Vec2::new(1.0, 1.0),
        theme::PLOT_ORIGIN
            + Vec2::new(
                theme::PLOT_COLUMNS as f32 + 1.0,
                theme::PLOT_ROWS as f32 + 1.0,
            ),
    )
}

pub fn is_starting_plot(column: u32, row: u32) -> bool {
    in_region(
        corner_of(column, row),
        theme::PLOT_ORIGIN,
        theme::PLOT_ORIGIN + Vec2::new(theme::PLOT_COLUMNS as f32, theme::PLOT_ROWS as f32),
    )
}

pub fn in_house(column: u32, row: u32) -> bool {
    let corner = corner_of(column, row);
    HouseKind::ALL.into_iter().any(|kind| {
        let house = kind.house();
        in_region(
            corner,
            house.origin,
            house.origin + Vec2::new(house.width as f32, house.depth as f32),
        )
    })
}

pub fn in_hollow_mouth(column: u32, row: u32) -> bool {
    in_region(corner_of(column, row), theme::HOLLOW_MIN, theme::HOLLOW_MAX)
}

pub fn natural_height(column: u32, row: u32) -> u32 {
    let corner = corner_of(column, row);
    let x = corner.x;
    let z = corner.y;
    let rolling = 2.6
        + 1.3 * (0.19 * x + 0.5).sin() * (0.15 * z - 0.3).cos()
        + 0.9 * (0.31 * x + 0.27 * z + 1.1).sin();
    let river = 2.2 * (-((z - river_center(x)).powi(2)) / 5.0).exp();
    let height = (rolling - river).round();
    (height as i32).clamp(theme::MIN_HEIGHT as i32, theme::MAX_HEIGHT as i32) as u32
}

pub fn on_path(column: u32, row: u32) -> bool {
    let middle = corner_of(column, row) + Vec2::new(0.5, 0.5);
    (middle.x - path_center(middle.y)).abs() < 1.0
}

pub fn is_rock(column: u32, row: u32) -> bool {
    let corner = corner_of(column, row);
    if in_region(corner, theme::HOLLOW_MIN, theme::HOLLOW_MAX) {
        return false;
    }
    in_region(corner, theme::CLIFF_MIN, theme::CLIFF_MAX)
        || in_region(corner, theme::QUARRY_MIN, theme::QUARRY_MAX)
        || in_region(corner, theme::RAMP_MIN, theme::RAMP_MAX)
}

pub fn shaped_height(column: u32, row: u32) -> (u32, bool) {
    let corner = corner_of(column, row);
    if in_region(corner, theme::VILLAGE_MIN, theme::VILLAGE_MAX) {
        return (theme::VILLAGE_HEIGHT, false);
    }
    if in_region(corner, theme::PEN_MIN, theme::PEN_MAX) {
        return (theme::PEN_HEIGHT, false);
    }
    if in_plot_plateau(column, row) {
        return (theme::PLOT_HEIGHT, false);
    }
    if in_region(corner, theme::HUT_MIN, theme::HUT_MAX) {
        return (theme::HUT_HEIGHT, false);
    }
    if in_region(corner, theme::HOLLOW_MIN, theme::HOLLOW_MAX) {
        return (theme::VILLAGE_HEIGHT, false);
    }
    if in_region(corner, theme::CLIFF_MIN, theme::CLIFF_MAX) {
        return (theme::CLIFF_HEIGHT, false);
    }
    if in_region(corner, theme::RAMP_MIN, theme::RAMP_MAX) {
        return (theme::RAMP_HEIGHT, false);
    }
    if in_region(corner, theme::QUARRY_MIN, theme::QUARRY_MAX) {
        return (theme::QUARRY_HEIGHT, false);
    }
    if in_region(corner, theme::DOCK_MIN, theme::DOCK_MAX) {
        return (theme::BRIDGE_HEIGHT, true);
    }
    if in_region(corner, theme::LAKE_MIN, theme::LAKE_MAX) {
        return (theme::MIN_HEIGHT, false);
    }
    let natural = natural_height(column, row);
    if in_region(corner, theme::MEADOW_MIN, theme::MEADOW_MAX) {
        return (
            natural.clamp(theme::MEADOW_MIN_HEIGHT, theme::MEADOW_MAX_HEIGHT),
            false,
        );
    }
    if natural <= theme::MIN_HEIGHT && on_path(column, row) {
        return (theme::BRIDGE_HEIGHT, true);
    }
    (natural, false)
}

pub fn zone_at(point: Vec2) -> Zone {
    if in_region(point, theme::CLIFF_MIN, theme::CLIFF_MAX) {
        Zone::Cliffs
    } else if in_region(point, theme::HOLLOW_MIN, theme::HOLLOW_MAX) {
        Zone::Quarry
    } else if in_region(point, theme::VILLAGE_MIN, theme::VILLAGE_MAX) {
        Zone::Village
    } else if in_region(point, theme::PEN_MIN, theme::PEN_MAX) {
        Zone::Pen
    } else if in_region(point, theme::QUARRY_MIN, theme::QUARRY_MAX)
        || in_region(point, theme::RAMP_MIN, theme::RAMP_MAX)
    {
        Zone::Quarry
    } else if in_region(point, theme::LAKE_MIN, theme::LAKE_MAX)
        || in_region(point, theme::HUT_MIN, theme::HUT_MAX)
    {
        Zone::Lake
    } else if in_region(point, theme::MEADOW_MIN, theme::MEADOW_MAX) {
        Zone::Meadow
    } else if in_region(point, theme::FOREST_MIN, theme::FOREST_MAX) {
        Zone::Forest
    } else {
        Zone::Farm
    }
}

pub fn is_path(terrain: &Terrain, column: u32, row: u32) -> bool {
    on_path(column, row)
        && height(terrain, column, row) > theme::MIN_HEIGHT
        && !is_rock(column, row)
}

pub fn is_tillable(terrain: &Terrain, farmland: &Farmland, column: u32, row: u32) -> bool {
    is_walkable(terrain, column, row)
        && !is_occupied(terrain, column, row)
        && !is_path(terrain, column, row)
        && !is_bridge(terrain, column, row)
        && !is_rock(column, row)
        && plot_at(farmland, column, row).is_none()
        && !in_region(
            corner_of(column, row),
            theme::VILLAGE_MIN,
            theme::VILLAGE_MAX,
        )
        && !in_region(corner_of(column, row), theme::PEN_MIN, theme::PEN_MAX)
}

pub fn plot_at(farmland: &Farmland, column: u32, row: u32) -> Option<usize> {
    farmland
        .plots
        .iter()
        .position(|plot| plot.column == column && plot.row == row)
}

pub fn lantern_spots(terrain: &Terrain) -> Vec<(f32, f32)> {
    let mut spots = Vec::new();
    let mut row = 2;
    while row < terrain.rows - 2 {
        let z = terrain.origin.y + row as f32 + 0.5;
        let x = path_center(z) + 1.5;
        if let Some((column, row_index)) = column_of(terrain, x, z)
            && height(terrain, column, row_index) > theme::MIN_HEIGHT
            && !is_bridge(terrain, column, row_index)
            && !is_rock(column, row_index)
        {
            spots.push((center(terrain, column, row_index).x, z));
        }
        row += 5;
    }
    for spot in theme::HOLLOW_LANTERNS {
        spots.push((spot.x, spot.y));
    }
    spots
}

pub fn ground_of(transform: &nightshade_scene::contract::LocalTransform) -> Vec2 {
    Vec2::new(transform.translation.x, transform.translation.z)
}

pub fn on_ground(terrain: &Terrain, x: f32, z: f32) -> nalgebra_glm::Vec3 {
    let height = height_at(terrain, x, z).unwrap_or(theme::MIN_HEIGHT);
    nalgebra_glm::Vec3::new(x, height as f32, z)
}
