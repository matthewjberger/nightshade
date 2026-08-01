use nalgebra_glm::Vec3;

#[derive(Default, Clone, Copy)]
pub struct Perch(pub Vec3);

#[derive(Default, Clone, Copy)]
pub struct Bob(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Phase(pub f32);

#[derive(Default, Clone, Copy)]
pub struct Spin(pub f32);
