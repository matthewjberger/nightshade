use crate::data;
use nalgebra_glm::Vec3;
use nightshade::prelude::Entity;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Paused,
    Over,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Won,
    Lost,
}

pub struct RoundState {
    pub score: u32,
    pub lives: u32,
    pub level: usize,
    pub chosen: usize,
    pub unlocked: usize,
    pub reached: usize,
    pub level_time: f32,
    pub bricks_standing: u32,
    pub serving: bool,
    pub rebuild_wanted: bool,
    pub outcome: Option<Outcome>,
    pub seed: u32,
}

impl Default for RoundState {
    fn default() -> Self {
        Self {
            score: 0,
            lives: data::STARTING_LIVES,
            level: 0,
            chosen: 0,
            unlocked: 0,
            reached: 0,
            level_time: 0.0,
            bricks_standing: 0,
            serving: true,
            rebuild_wanted: true,
            outcome: None,
            seed: 0x9e37_79b9,
        }
    }
}

pub struct Juice {
    pub shake: f32,
    pub tint: Vec3,
    pub tint_target: Vec3,
    pub zoom: f32,
    pub time_scale: f32,
    pub slow_hold: f32,
    pub trail_debt: f32,
}

impl Default for Juice {
    fn default() -> Self {
        Self {
            shake: 0.0,
            tint: Vec3::zeros(),
            tint_target: Vec3::zeros(),
            zoom: 1.0,
            time_scale: 1.0,
            slow_hold: 0.0,
            trail_debt: 0.0,
        }
    }
}

#[derive(Default)]
pub struct PadState {
    pub focus: usize,
    pub repeat: f32,
}

#[derive(Default, Clone, Copy)]
pub struct UiHandles {
    pub readout: Option<Entity>,
    pub level_button: Entity,
    pub play_button: Entity,
    pub title_quit_button: Entity,
    pub resume_button: Entity,
    pub restart_button: Entity,
    pub pause_title_button: Entity,
    pub again_button: Entity,
    pub over_title_button: Entity,
    pub over_heading: Entity,
    pub over_summary: Entity,
}

#[derive(Default)]
pub struct BreakoutAdvancedResources {
    pub round: RoundState,
    pub juice: Juice,
    pub pad: PadState,
    pub ui: UiHandles,
}
