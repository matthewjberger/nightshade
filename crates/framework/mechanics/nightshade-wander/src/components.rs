use nalgebra_glm::Vec3;

#[derive(Default, Clone, Copy)]
pub struct Home(pub Vec3);

#[derive(Default, Clone, Copy)]
pub struct Wandering(pub Vec3);

#[derive(Default, Clone, Copy)]
pub struct Roaming(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Resting(pub f32);
