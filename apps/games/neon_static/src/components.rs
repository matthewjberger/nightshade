use nalgebra_glm::Vec2;

#[derive(Default, Clone, Copy, Debug)]
pub struct Angle(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Speed(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Cooldown(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Spin(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Life(pub f32);
