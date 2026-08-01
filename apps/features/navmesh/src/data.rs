use crate::commands::{NoiseConfig, NoiseType, TerrainConfig};
use nightshade::prelude::*;

pub const FOX_MODEL: &[u8] = include_bytes!("../../../../assets/gltf/fox.glb");
pub const FOX_SCALE: f32 = 0.01;
pub const HDR_SKYBOX: &[u8] = include_bytes!("../../../../assets/sky/moonrise.hdr");

pub const FIRST_NAMES: &[&str] = &[
    "Rusty", "Maple", "Bramble", "Hazel", "Fern", "Copper", "Willow", "Ash", "Clover", "Sage",
    "Juniper", "Ember", "Moss", "Birch", "Reed", "Flint",
];

pub const LAST_NAMES: &[&str] = &[
    "Swift", "Whisker", "Paw", "Tail", "Ear", "Nose", "Fur", "Bark", "Leaf", "Stone", "Brook",
    "Glen", "Hill", "Vale", "Frost", "Dawn",
];

pub const DEFAULT_AGENT_SPEED: f32 = 5.0;
pub const IDLE_SPEED_THRESHOLD: f32 = 0.25;
pub const STILL_FRAMES_FOR_IDLE: u32 = 3;
pub const RUN_DISTANCE_THRESHOLD: f32 = 8.0;
pub const RUN_CLIP_GROUND_SPEED: f32 = DEFAULT_AGENT_SPEED;
pub const WALK_CLIP_GROUND_SPEED: f32 = DEFAULT_AGENT_SPEED / 1.5;
pub const PLAUSIBLE_STEP_MULTIPLE: f32 = 3.0;

pub const TREE_COUNT: usize = 40;
pub const INITIAL_FOXES: [(f32, f32); 3] = [(-5.0, -5.0), (5.0, -5.0), (-5.0, 5.0)];

pub fn terrain_config() -> TerrainConfig {
    TerrainConfig {
        width: 80.0,
        depth: 80.0,
        resolution_x: 64,
        resolution_z: 64,
        height_scale: 3.0,
        noise: NoiseConfig {
            seed: 42,
            frequency: 0.02,
            octaves: 4,
            lacunarity: 2.0,
            persistence: 0.45,
            noise_type: NoiseType::Perlin,
        },
        uv_scale: [10.0, 10.0],
    }
}

#[derive(Default)]
pub struct AnimationIndices {
    pub survey: Option<usize>,
    pub walk: Option<usize>,
    pub run: Option<usize>,
}

pub struct TreeObstacle {
    pub position: Vec3,
    pub radius: f32,
    pub height: f32,
}
