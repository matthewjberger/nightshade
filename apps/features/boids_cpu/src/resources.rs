use crate::data::FlockGrid;
use nightshade::prelude::{Entity, Instant};
use nightshade_cli::clap;
use nightshade_render::components::mesh::InstanceCustomData;
use std::sync::Arc;

pub const INITIAL_BOID_COUNT: usize = 100_000;

#[derive(clap::Args)]
pub struct BoidsArguments {
    #[arg(
        value_name = "COUNT",
        default_value_t = INITIAL_BOID_COUNT,
        help = "How many boids to start with"
    )]
    pub count: usize,
}

impl Default for BoidsArguments {
    fn default() -> Self {
        Self {
            count: INITIAL_BOID_COUNT,
        }
    }
}

pub const BOID_STEP: usize = 50_000;

pub const MIN_BOID_COUNT: usize = BOID_STEP;

const REFERENCE_COUNT: f32 = 100_000.0;

const REFERENCE_BOUNDS: f32 = 100.0;

pub fn bounds_for(count: usize) -> f32 {
    REFERENCE_BOUNDS * (count as f32 / REFERENCE_COUNT).cbrt()
}

pub const SEARCH_RADIUS: f32 = 5.0;

pub const GRID_MARGIN: f32 = SEARCH_RADIUS * 6.0;

pub const CANDIDATE_CAP: usize = 48;

#[derive(Default)]
pub struct GridBuilder {
    pub grid: Arc<FlockGrid>,
    pub bounds: f32,
}

pub struct Integration {
    pub delta_time: f32,
    pub regions: usize,
    pub max_speed: f32,
    pub bounds: f32,
    pub turn_strength: f32,
}

impl Default for Integration {
    fn default() -> Self {
        Self {
            delta_time: 0.0,
            regions: 1,
            max_speed: 26.0,
            bounds: bounds_for(INITIAL_BOID_COUNT),
            turn_strength: 40.0,
        }
    }
}

pub const SEPARATION_WEIGHT: f32 = 3.2;
pub const ALIGNMENT_WEIGHT: f32 = 1.2;
pub const COHESION_WEIGHT: f32 = 0.9;

#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub rate_readout: Entity,
    pub count_readout: Entity,
    pub step_readout: Entity,
    pub sync_readout: Entity,
    pub region_readout: Entity,
    pub other_readout: Entity,
    pub batch_readout: Entity,
    pub overlay_buttons: [Entity; 4],
}

#[derive(Default)]
pub struct BoidsResources {
    pub flock: std::sync::Mutex<nightshade::ecs::Schedule<nightshade::ecs::World>>,
    pub boids: Vec<Entity>,
    pub requested_count: usize,
    pub draw: Option<Entity>,
    pub instances: Vec<nightshade_render::components::InstanceTransform>,
    pub tints: Vec<InstanceCustomData>,
    pub tinted_by_region: bool,
    pub stage_clock: Option<Instant>,
    pub stage_seconds: [f64; StageMark::COUNT],
    pub lines: Option<Entity>,
    pub batches: Vec<std::ops::Range<usize>>,
    pub show_headings: bool,
    pub show_bounds: bool,
    pub show_neighbors: bool,
    pub show_boids: bool,
    pub step_seconds: f64,
    pub sync_seconds: f64,
    pub step_count: u32,
    pub reported_milliseconds: f64,
    pub reported_sync_milliseconds: f64,
    pub reported_imbalance: f32,
    pub reported_regions: usize,
    pub reported_frame_milliseconds: f64,
    pub frame_seconds: f64,
    pub report_countdown: f32,
    pub hud: HudHandles,
    pub hud_count: usize,
    pub hud_step: f64,
    pub hud_sync: f64,
    pub hud_imbalance: f32,
    pub hud_frame: f64,
    pub hud_batches: Vec<std::ops::Range<usize>>,
    pub hud_overlays: [bool; 4],
    pub hud_rate: u32,
}

#[derive(Clone, Copy)]
pub enum StageMark {
    First,
    Update,
    FrameStart,
    FrameUpdate,
    FramePostUpdate,
    Render,
    Last,
}

impl StageMark {
    pub const COUNT: usize = 7;

    pub const NAMED: [(StageMark, &'static str); Self::COUNT] = [
        (StageMark::First, "first"),
        (StageMark::Update, "update"),
        (StageMark::FrameStart, "start"),
        (StageMark::FrameUpdate, "frame"),
        (StageMark::FramePostUpdate, "post"),
        (StageMark::Render, "render"),
        (StageMark::Last, "last"),
    ];
}
