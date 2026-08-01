use crate::data;
use crate::resources::{Outcome, RoundState, Screen, UiHandles};
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::Entity;

pub fn level_rows(level: usize) -> &'static [&'static str] {
    data::LEVELS[level.min(data::LEVELS.len() - 1)]
}

pub fn brick_armor(level: usize, column: u32, row: u32) -> Option<u32> {
    let line = level_rows(level).get(row as usize)?;
    match line.as_bytes().get(column as usize)? {
        b'1' => Some(0),
        b'2' => Some(1),
        b'3' => Some(2),
        _ => None,
    }
}

pub fn level_brick_count(level: usize) -> u32 {
    let mut total = 0;
    for row in 0..data::BRICK_ROWS {
        for column in 0..data::BRICK_COLUMNS {
            if brick_armor(level, column, row).is_some() {
                total += 1;
            }
        }
    }
    total
}

pub fn brick_center(column: u32, row: u32) -> Vec2 {
    let span = (data::BRICK_COLUMNS - 1) as f32 * data::BRICK_SPACING_X;
    Vec2::new(
        column as f32 * data::BRICK_SPACING_X - span * 0.5,
        data::BRICK_TOP_Y - row as f32 * data::BRICK_SPACING_Y,
    )
}

pub fn row_of(height: f32) -> u32 {
    let step = ((data::BRICK_TOP_Y - height) / data::BRICK_SPACING_Y).round();
    step.clamp(0.0, (data::BRICK_ROWS - 1) as f32) as u32
}

pub fn row_color(row: u32) -> [f32; 4] {
    data::ROW_COLORS[(row as usize).min(data::ROW_COLORS.len() - 1)]
}

pub fn row_points(row: u32) -> u32 {
    data::ROW_POINTS[(row as usize).min(data::ROW_POINTS.len() - 1)]
}

pub fn tier_color(row: u32, armor: u32) -> [f32; 4] {
    let base = row_color(row);
    let shade = data::TIER_SHADE[(armor as usize).min(data::TIER_SHADE.len() - 1)];
    [base[0] * shade, base[1] * shade, base[2] * shade, base[3]]
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

pub fn level_par(level: usize) -> f32 {
    data::LEVEL_PAR[level.min(data::LEVEL_COUNT - 1)]
}

pub fn beat_par(round: &RoundState) -> bool {
    round.level_time <= level_par(round.level)
}

pub fn level_label(chosen: usize) -> String {
    format!("Level {}", chosen + 1)
}

pub fn readout_text(round: &RoundState) -> String {
    format!(
        "Level {}     Score {}     Par {:.0} of {:.0}     Lives {}     Bricks {}",
        round.level + 1,
        round.score,
        round.level_time,
        level_par(round.level),
        round.lives,
        round.bricks_standing
    )
}

pub fn outcome_heading(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Won => "All levels cleared",
        Outcome::Lost => "Game over",
    }
}

pub fn outcome_summary(round: &RoundState) -> String {
    format!(
        "Score {}     Reached level {}",
        round.score,
        round.reached + 1
    )
}

pub fn shake_offset(amount: f32, clock: f32) -> Vec2 {
    let reach = amount * data::SHAKE_REACH;
    Vec2::new(
        (clock * 57.0).sin() * reach + (clock * 23.0).cos() * reach * 0.4,
        (clock * 43.0).cos() * reach + (clock * 31.0).sin() * reach * 0.4,
    )
}

pub fn zoom_target(bricks_standing: u32) -> f32 {
    if bricks_standing == 0 || bricks_standing > data::ZOOM_FROM {
        return 1.0;
    }
    let closeness = 1.0 - (bricks_standing - 1) as f32 / data::ZOOM_FROM as f32;
    1.0 + (data::ZOOM_NEAR - 1.0) * closeness
}

pub fn tint_of(color: [f32; 4]) -> Vec3 {
    Vec3::new(color[0], color[1], color[2]) * data::TINT_STRENGTH
}

pub fn background(tint: Vec3) -> [f32; 4] {
    [
        data::CLEAR_COLOR[0] + tint.x,
        data::CLEAR_COLOR[1] + tint.y,
        data::CLEAR_COLOR[2] + tint.z,
        1.0,
    ]
}

pub fn ball_shape(velocity: Vec2, squash: f32) -> Vec2 {
    let haste = (velocity.magnitude() / data::BALL_SPEED).clamp(0.0, 1.4);
    let along = 1.0 + haste * data::STRETCH_GAIN - squash * data::SQUASH_GAIN;
    let across = 1.0 - haste * data::STRETCH_GAIN * 0.5 + squash * data::SQUASH_GAIN;
    Vec2::new(along.max(0.2), across.max(0.2))
}

pub fn pop_scale(remaining: f32) -> f32 {
    let progress = (1.0 - remaining / data::POP_SECONDS).clamp(0.0, 1.0);
    (1.0 + data::POP_SWELL * progress) * (1.0 - progress * progress * progress)
}

pub fn grow_scale(remaining: f32) -> f32 {
    if remaining > data::GROW_SECONDS {
        return 0.0;
    }
    let progress = (1.0 - remaining / data::GROW_SECONDS).clamp(0.0, 1.0);
    let back = progress - 1.0;
    (1.0 + 2.70158 * back * back * back + 1.70158 * back * back).max(0.0)
}

pub fn fade_scale(remaining: f32) -> f32 {
    (remaining / data::TRAIL_LIFE).clamp(0.0, 1.0)
}

pub fn screen_buttons(handles: &UiHandles, screen: Screen) -> Vec<Entity> {
    match screen {
        Screen::Title => vec![
            handles.level_button,
            handles.play_button,
            handles.title_quit_button,
        ],
        Screen::Paused => vec![
            handles.resume_button,
            handles.restart_button,
            handles.pause_title_button,
        ],
        Screen::Over => vec![handles.again_button, handles.over_title_button],
        Screen::Playing => Vec::new(),
    }
}

fn sign(value: f32) -> f32 {
    if value < 0.0 { -1.0 } else { 1.0 }
}
