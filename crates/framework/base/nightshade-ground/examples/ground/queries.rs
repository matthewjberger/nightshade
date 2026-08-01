use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::Storage;

pub fn hill_height(_storage: &Storage, spot: Vec2) -> Option<f32> {
    Some(height_of(spot.x, spot.y))
}

pub fn height_of(across: f32, along: f32) -> f32 {
    let wave = (across * theme::WAVE).sin() + (along * theme::WAVE * 0.7).cos();
    (wave + 2.0) * 0.25 * theme::LIFT
}

pub fn solid_here(_storage: &Storage, point: Vec3) -> bool {
    point.x.abs() <= theme::BLOCK && point.z.abs() <= theme::BLOCK && point.y <= theme::BLOCK * 2.0
}
