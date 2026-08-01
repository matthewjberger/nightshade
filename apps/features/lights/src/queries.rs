use crate::data::{ARENA_SIZE, NUM_LIGHTS};
use nightshade::prelude::*;

pub fn light_position(index: usize, seconds: f32) -> Vec3 {
    let phase = (index as f32 / NUM_LIGHTS as f32) * std::f32::consts::TAU;
    let speed = 0.3 + (index as f32 * 0.01) % 0.2;
    let turned = phase + seconds * speed;
    let radius = (index as f32 / NUM_LIGHTS as f32) * (ARENA_SIZE * 0.8)
        + 5.0
        + (seconds * 0.5 + phase).sin() * 3.0;
    let height = 2.0 + (index as f32 * 0.1) % 3.0 + (seconds * 2.0 + phase * 2.0).sin() * 1.5;
    Vec3::new(radius * turned.cos(), height, radius * turned.sin())
}
