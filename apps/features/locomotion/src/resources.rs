use nightshade::prelude::*;

#[derive(Default)]
pub struct Character {
    pub body: Option<Entity>,
    pub model: Option<Entity>,
    pub camera: Option<Entity>,
    pub asked: Option<nightshade_load::LoadHandle>,
    pub facing: f32,
    pub velocity: Vec3,
    pub forward_parameter: f32,
    pub lateral_parameter: f32,
    pub airborne: bool,
    pub locked: bool,
    pub facing_error: f32,
    pub turn_weight: f32,
    pub turn_clip: Option<usize>,
    pub jump_weight: f32,
    pub index: usize,
    pub held_clips: Option<nightshade_animation::contract::ClipSet>,
    pub swapping: bool,
}

#[derive(Default)]
pub struct Arena {
    pub blocks: Vec<(Vec3, Vec3)>,
    pub pickups: Vec<Entity>,
    pub collected: usize,
    pub total: usize,
}

#[derive(Default)]
pub struct Readout {
    pub root: Option<Entity>,
    pub speed_text: Option<Entity>,
    pub blend_text: Option<Entity>,
    pub clip_text: Option<Entity>,
    pub pad_text: Option<Entity>,
    pub shown: Option<(i32, i32, i32, i32, i32)>,
    pub clips_shown: Option<String>,
    pub pad_shown: Option<String>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Paused,
}

#[derive(Default, Clone, Copy)]
pub struct ScreenHandles {
    pub play_button: Entity,
    pub title_quit_button: Entity,
    pub resume_button: Entity,
    pub pause_quit_button: Entity,
}

#[derive(Default)]
pub struct LocomotionResources {
    pub screens: ScreenHandles,
    pub character: Character,
    pub readout: Readout,
    pub arena: Arena,
    pub walking: bool,
}
