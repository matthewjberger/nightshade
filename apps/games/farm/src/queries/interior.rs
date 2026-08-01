use crate::resources::Home;
use nalgebra_glm::Vec2;

pub fn center(home: &Home, column: u32, row: u32) -> Vec2 {
    Vec2::new(
        column as f32 + 0.5 - home.columns as f32 / 2.0,
        row as f32 + 0.5 - home.rows as f32 / 2.0,
    )
}

pub fn cell_of(home: &Home, spot: Vec2) -> Option<(u32, u32)> {
    let column = spot.x + home.columns as f32 / 2.0;
    let row = spot.y + home.rows as f32 / 2.0;
    if column < 0.0 || row < 0.0 {
        return None;
    }
    let (column, row) = (column as u32, row as u32);
    if column >= home.columns || row >= home.rows {
        return None;
    }
    Some((column, row))
}

pub fn is_blocked(home: &Home, column: u32, row: u32) -> bool {
    home.blocked
        .get((row * home.columns + column) as usize)
        .copied()
        .unwrap_or(true)
}

pub fn landing_spot(home: &Home) -> Vec2 {
    for step in 1..=home.rows {
        for side in [0.0, -1.0, 1.0, -2.0, 2.0] {
            let spot = home.exit + Vec2::new(side, step as f32);
            if can_step(home, spot) {
                return spot;
            }
        }
    }
    home.exit
}

pub fn can_step(home: &Home, spot: Vec2) -> bool {
    let half_width = home.columns as f32 / 2.0 - 0.3;
    let half_depth = home.rows as f32 / 2.0 - 0.3;
    if spot.x.abs() > half_width || spot.y.abs() > half_depth {
        return false;
    }
    match cell_of(home, spot) {
        Some((column, row)) => !is_blocked(home, column, row),
        None => false,
    }
}
