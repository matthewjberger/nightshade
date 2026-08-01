use nalgebra_glm::Vec3;

#[derive(Default, Clone, Copy)]
pub struct Trauma(pub f32);

#[derive(Default, Clone, Copy)]
pub struct ShakeOffset(pub Vec3);

#[derive(Default, Clone, Copy)]
pub struct ShakeTurn(pub f32);
