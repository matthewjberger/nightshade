use crate::components::{Layers, Seed};
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::ecs::Storage;
use nightshade_noise::layered;

pub fn hill_height(storage: &Storage, spot: Vec2) -> Option<f32> {
    let (seed, layers) = shape_of(storage)?;
    Some(height_at(spot.x, spot.y, seed, layers))
}

pub fn shape_of(storage: &Storage) -> Option<(u32, u32)> {
    storage
        .query_ref::<(&Seed, &Layers)>()
        .iter()
        .next()
        .map(|(_entity, (seed, layers))| (seed.0, layers.0))
}

pub fn height_at(across: f32, along: f32, seed: u32, layers: u32) -> f32 {
    let raw = layered(
        across * theme::SCALE,
        along * theme::SCALE,
        seed,
        layers,
        theme::FADE,
    );
    raw * theme::LIFT
}

pub fn tint_for(height: f32) -> [f32; 4] {
    let share = (height / theme::LIFT).clamp(0.0, 1.0);
    let mix = |low: f32, high: f32| low + (high - low) * share;
    [
        mix(theme::LOW_TINT[0], theme::HIGH_TINT[0]),
        mix(theme::LOW_TINT[1], theme::HIGH_TINT[1]),
        mix(theme::LOW_TINT[2], theme::HIGH_TINT[2]),
        1.0,
    ]
}
