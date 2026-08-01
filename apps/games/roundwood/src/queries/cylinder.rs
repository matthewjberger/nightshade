use crate::data;
use nalgebra_glm::{Quat, Vec2, Vec3};

pub fn angle(wrap: f32) -> f32 {
    wrap * std::f32::consts::TAU / data::CIRCUMFERENCE
}

pub fn across_of(column: f32) -> f32 {
    column - data::ACROSS as f32 / 2.0
}

pub fn surface(across: f32, wrap: f32, lift: f32) -> Vec3 {
    let angle = angle(wrap);
    let radius = data::RADIUS + lift;
    Vec3::new(
        across_of(across),
        radius * angle.cos(),
        radius * angle.sin(),
    )
}

pub fn place(town: Vec2, lift: f32) -> Vec3 {
    surface(town.x, town.y, lift)
}

pub fn tilt(wrap: f32) -> Quat {
    nalgebra_glm::quat_angle_axis(angle(wrap), &Vec3::x())
}

pub fn spin(wrap: f32) -> Quat {
    nalgebra_glm::quat_angle_axis(-angle(wrap), &Vec3::x())
}

pub fn wrap(around: f32) -> f32 {
    around.rem_euclid(data::CIRCUMFERENCE)
}

pub fn ring_delta(from: f32, to: f32) -> f32 {
    let half = data::CIRCUMFERENCE / 2.0;
    (to - from + half).rem_euclid(data::CIRCUMFERENCE) - half
}

pub fn town_delta(from: Vec2, to: Vec2) -> Vec2 {
    Vec2::new(to.x - from.x, ring_delta(from.y, to.y))
}

pub fn town_distance(from: Vec2, to: Vec2) -> f32 {
    town_delta(from, to).magnitude()
}

pub fn arc_at(lift: f32) -> f32 {
    2.0 * (data::RADIUS + lift) * (data::STEP_ANGLE / 2.0).tan()
}
