use nalgebra_glm::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Noise {
    pub at: Vec3,
    pub loudness: f32,
}
