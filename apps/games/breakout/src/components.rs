use nalgebra_glm::Vec2;

#[derive(Default, Clone, Copy)]
pub struct Speed(pub f32);

#[derive(Default, Clone, Copy)]
pub struct HalfExtent(pub Vec2);

#[derive(Default, Clone, Copy)]
pub struct Velocity(pub Vec2);

#[derive(Default, Clone, Copy)]
pub struct Radius(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Points(pub u32);
