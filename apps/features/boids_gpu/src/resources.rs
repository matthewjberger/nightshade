use crate::data::INITIAL_BOID_COUNT;
use nightshade::prelude::Entity;
use nightshade_cli::clap;

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

#[derive(Clone, Copy)]
pub struct FlockSettings {
    pub delta_time: f32,
    pub bounds: f32,
    pub search_radius: f32,
    pub max_speed: f32,
    pub separation: f32,
    pub alignment: f32,
    pub cohesion: f32,
    pub turn_strength: f32,
    pub count: u32,
    pub visible: bool,
}

impl Default for FlockSettings {
    fn default() -> Self {
        Self {
            delta_time: 0.0,
            bounds: 120.0,
            search_radius: 5.0,
            max_speed: 26.0,
            separation: 14.0,
            alignment: 0.08,
            cohesion: 0.9,
            turn_strength: 0.6,
            count: 0,
            visible: true,
        }
    }
}

#[derive(Default)]
pub struct BoidsGpuResources {
    pub count: usize,
    pub visible: bool,
    pub seeded: bool,
    pub count_label: Option<Entity>,
    pub frame_label: Option<Entity>,
    pub visible_label: Option<Entity>,
    pub frame_seconds: f64,
    pub frame_count: u32,
    pub report_countdown: f32,
    pub reported_frame_milliseconds: f64,
}
