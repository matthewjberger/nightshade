use nalgebra_glm::Vec3;

#[derive(Default, Clone)]
pub struct Position(pub Vec3);

#[derive(Default, Clone)]
pub struct Velocity(pub Vec3);

#[derive(Default, Clone)]
pub struct Shade(pub [f32; 4]);
