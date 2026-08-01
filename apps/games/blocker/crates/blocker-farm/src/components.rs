use nalgebra_glm::IVec3;

#[derive(Default, Clone, Copy, Debug)]
pub struct Block(pub IVec3);

#[derive(Default, Clone, Copy, Debug)]
pub struct Progress(pub f32);

#[derive(Default, Clone, Copy, Debug)]
pub struct Stage(pub usize);

#[derive(Default, Clone, Copy, Debug)]
pub struct Watered(pub bool);
