use nalgebra_glm::Vec3;
use nightshade_render::data::geometry::Mesh;

#[derive(Clone, Copy)]
pub struct Bump {
    pub toward: Vec3,
    pub weight: f32,
}

pub struct SculptJob {
    pub slot: usize,
    pub seed: u64,
    pub era: u32,
}

pub struct SculptWork {
    pub slot: usize,
    pub mesh: Mesh,
    pub era: u32,
}

#[derive(Default)]
pub struct Chisel {
    pub bumps: Vec<Bump>,
    pub reach: Vec<f32>,
    pub place: Vec<Vec3>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    #[default]
    Empty,
    Asked,
    Standing,
}
