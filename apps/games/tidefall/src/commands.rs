use crate::data::PAD_CLEARANCE;
use crate::resources::TidefallResources;
use nightshade::prelude::*;

pub fn record_landing(resources: &mut TidefallResources) {
    resources.landings += 1;
}

pub fn draw_direction(roll: &mut fastrand::Rng) -> Vec3 {
    let height = roll.f32() * 2.0 - 1.0;
    let angle = roll.f32() * std::f32::consts::TAU;
    let ring = (1.0 - height * height).max(0.0).sqrt();
    Vec3::new(ring * angle.cos(), height, ring * angle.sin())
}

pub fn draw_clear_direction(roll: &mut fastrand::Rng, pad: Option<Vec3>) -> Vec3 {
    let mut direction = draw_direction(roll);
    while let Some(pad) = pad
        && direction.dot(&pad) > PAD_CLEARANCE
    {
        direction = draw_direction(roll);
    }
    direction
}

pub fn draw_near(roll: &mut fastrand::Rng, around: Vec3, spread: f32) -> Vec3 {
    (around + draw_direction(roll) * spread).normalize()
}

pub fn draw_beside(roll: &mut fastrand::Rng, around: Vec3, angle: f32) -> Vec3 {
    let mut tangent = draw_direction(roll).cross(&around);
    while tangent.magnitude() < 1.0e-3 {
        tangent = draw_direction(roll).cross(&around);
    }
    let tangent = tangent.normalize();
    (around * angle.cos() + tangent * angle.sin()).normalize()
}

pub fn draw_between(roll: &mut fastrand::Rng, low: f32, high: f32) -> f32 {
    low + roll.f32() * (high - low)
}
