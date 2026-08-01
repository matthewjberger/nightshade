use crate::data;
use crate::resources::RoundState;
use nalgebra_glm::Vec2;

pub fn brick_center(column: u32, row: u32) -> Vec2 {
    let span = (data::BRICK_COLUMNS - 1) as f32 * data::BRICK_SPACING_X;
    Vec2::new(
        column as f32 * data::BRICK_SPACING_X - span * 0.5,
        data::BRICK_TOP_Y - row as f32 * data::BRICK_SPACING_Y,
    )
}

pub fn row_color(row: u32) -> [f32; 4] {
    data::ROW_COLORS[row as usize]
}

pub fn row_points(row: u32) -> u32 {
    data::ROW_POINTS[row as usize]
}

pub fn serve_position(paddle_x: f32) -> Vec2 {
    Vec2::new(paddle_x, data::PADDLE_Y + data::SERVE_HEIGHT)
}

pub fn bounce_direction(offset: f32) -> Vec2 {
    let angle = offset.clamp(-1.0, 1.0) * data::MAX_BOUNCE_RADIANS;
    Vec2::new(angle.sin(), angle.cos())
}

pub fn separation(center: Vec2, radius: f32, box_center: Vec2, half_extent: Vec2) -> Option<Vec2> {
    let delta = center - box_center;
    let clamped = Vec2::new(
        delta.x.clamp(-half_extent.x, half_extent.x),
        delta.y.clamp(-half_extent.y, half_extent.y),
    );
    let offset = delta - clamped;
    let distance = offset.magnitude();
    if distance > f32::EPSILON {
        if distance >= radius {
            return None;
        }
        return Some(offset / distance);
    }

    let reach_x = half_extent.x + radius - delta.x.abs();
    let reach_y = half_extent.y + radius - delta.y.abs();
    if reach_x < reach_y {
        Some(Vec2::new(sign(delta.x), 0.0))
    } else {
        Some(Vec2::new(0.0, sign(delta.y)))
    }
}

pub fn axis_normal(normal: Vec2) -> Vec2 {
    if normal.x.abs() >= normal.y.abs() {
        Vec2::new(sign(normal.x), 0.0)
    } else {
        Vec2::new(0.0, sign(normal.y))
    }
}

pub fn readout_text(round: &RoundState) -> String {
    format!(
        "{}   Score {}   Lives {}   Bricks {}",
        data::TITLE,
        round.score,
        round.lives,
        round.bricks_standing
    )
}

fn sign(value: f32) -> f32 {
    if value < 0.0 { -1.0 } else { 1.0 }
}
