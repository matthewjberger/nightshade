use fastrand::Rng;
use nalgebra_glm::Vec2;

#[derive(Default, Clone, Copy, Debug)]
pub struct Speed(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Target(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Rest(pub f32);

#[derive(Default, Clone, Debug)]
pub struct Roam(pub Rng);

#[derive(Default, Clone, Copy, Debug)]
pub struct Spin(pub f32);
