use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::ecs::Storage;

pub fn on_the_field(across: f32, along: f32) -> bool {
    across.abs() <= theme::EDGE && along.abs() <= theme::EDGE
}

pub fn hill_height(_storage: &Storage, spot: Vec2) -> Option<f32> {
    on_the_field(spot.x, spot.y).then(|| height_of(spot.x, spot.y))
}

pub fn height_of(across: f32, along: f32) -> f32 {
    let wave = (across * theme::WAVE).sin() + (along * theme::WAVE * 0.8).cos();
    (wave + 2.0) * 0.25 * theme::LIFT
}
