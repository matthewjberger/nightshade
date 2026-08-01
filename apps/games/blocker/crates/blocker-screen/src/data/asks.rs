use nalgebra_glm::{Vec2, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct BeastWanted {
    pub place: Vec3,
    pub heading: f32,
    pub kind: usize,
    pub lair: Vec2,
}
