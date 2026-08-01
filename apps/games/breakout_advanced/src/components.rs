use nalgebra_glm::Vec2;

#[derive(Default, Clone, Copy, Debug)]
pub struct Speed(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct HalfExtent(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Default, Clone, Copy, Debug)]
pub struct Radius(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Points(pub u32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Lean(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct LeanRate(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Squash(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Pop(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Grow(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Fade(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Armor(pub u32);
