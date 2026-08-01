use crate::data::PAD_COUNT;
use nightshade::prelude::{Entity, Vec2};
use nightshade_load::LoadHandle;
use nightshade_prefab::Prefab;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Paused,
}

#[derive(Default)]
pub struct HairpinResources {
    pub screen: Screen,
    pub lap: Lap,
    pub models: Models,
    pub track: Track,
    pub ui: Ui,
}

#[derive(Default)]
pub struct Lap {
    pub current: f32,
    pub best: Option<f32>,
    pub count: u32,
    pub ahead_of_line: bool,
}

#[derive(Default)]
pub struct Models {
    pub handles: Vec<LoadHandle>,
    pub prefabs: Vec<Option<Prefab>>,
}

#[derive(Default)]
pub struct Track {
    pub built: bool,
    pub car: Option<Entity>,
    pub camera: Option<Entity>,
    pub limits: TrackLimits,
}

#[derive(Default, Clone, Copy)]
pub struct Bounds {
    pub minimum: Vec2,
    pub maximum: Vec2,
}

#[derive(Default, Clone, Copy)]
pub struct TrackLimits {
    pub outer: Bounds,
    pub inner: Bounds,
}

#[derive(Default, Clone, Copy)]
pub struct Ui {
    pub hud: Option<Entity>,
    pub title: Option<Entity>,
    pub pause: Option<Entity>,
    pub start_button: Option<Entity>,
    pub exit_button: Option<Entity>,
    pub resume_button: Option<Entity>,
    pub restart_button: Option<Entity>,
    pub quit_button: Option<Entity>,
    pub pads: [Option<Entity>; PAD_COUNT],
    pub pause_pad: Option<Entity>,
}
