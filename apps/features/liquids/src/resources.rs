use nightshade::prelude::Entity;
pub use nightshade_voxels::prelude::{VoxelSettings, VoxelViewer, VoxelWorld};
use noise::Perlin;

pub struct Digging;

#[derive(Default)]
pub struct Seed(pub u32);

pub struct Noises {
    pub seed: u32,
    pub land: Perlin,
    pub heat: Perlin,
}

impl Default for Noises {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Noises {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            land: Perlin::new(seed),
            heat: Perlin::new(seed.wrapping_add(1)),
        }
    }
}

#[derive(Default)]
pub struct Watcher {
    pub camera: Option<Entity>,
}
