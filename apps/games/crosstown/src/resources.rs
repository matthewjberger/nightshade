use nalgebra_glm::Vec3;
use nightshade::prelude::Entity;
use nightshade_cli::clap;
use nightshade_load::LoadHandle;
use nightshade_prefab::Prefab;
use nightshade_sprite::prelude::SpriteRegion;

#[derive(clap::Args, Default)]
pub struct Arguments {
    #[arg(long, help = "Open on the street instead of the title card")]
    pub play: bool,
    #[arg(long, help = "Open in arcade mode instead of the story")]
    pub arcade: bool,
    #[arg(long, value_name = "INDEX", help = "Open on this story stage")]
    pub stage: Option<usize>,
    #[arg(long, value_name = "INDEX", help = "Open on this wave of the stage")]
    pub wave: Option<usize>,
    #[arg(
        long,
        help = "Open with the floor already clear, so the GO arrow shows"
    )]
    pub cleared: bool,
    #[arg(long, value_name = "COUNT", help = "How many thugs the wave holds")]
    pub thugs: Option<u32>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Screen {
    #[default]
    Title,
    Play,
    Pause,
    Over,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    #[default]
    Story,
    Arcade,
}

#[derive(Default)]
pub struct Sheets {
    pub pages: Vec<SpriteRegion>,
    pub shadow: SpriteRegion,
    pub weapons: Vec<SpriteRegion>,
    pub pickups: Vec<SpriteRegion>,
}

#[derive(Default)]
pub struct Models {
    pub handles: Vec<LoadHandle>,
    pub prefabs: Vec<Option<Prefab>>,
    pub ready: bool,
}

#[derive(Clone, Copy, Default)]
pub struct Blow {
    pub at: Vec3,
    pub facing: f32,
    pub step: u32,
    pub reach: f32,
    pub damage: f32,
    pub push: f32,
    pub down: bool,
}

#[derive(Default)]
pub struct Ring {
    pub hero: Entity,
    pub camera: Entity,
    pub feet: Vec3,
    pub hero_health: f32,
    pub hero_blow: Option<Blow>,
    pub thug_damage: f32,
    pub thug_push: f32,
    pub hit_stop: f32,
    pub shake: f32,
    pub camera_x: f32,
    pub alive: u32,
    pub left: [u32; crate::data::KIND_COUNT],
    pub delay: f32,
    pub from_right: bool,
    pub cleared: bool,
    pub flash: f32,
    pub mark: f32,
    pub mark_name: &'static str,
    pub chief: Entity,
    pub chief_name: &'static str,
    pub chief_share: f32,
    pub gate: Vec<Entity>,
    pub mode: Mode,
    pub stage: usize,
    pub wave: usize,
    pub waves: usize,
    pub round: u32,
    pub score: u32,
    pub built: bool,
    pub opened: bool,
    pub last_wave: bool,
    pub piece: u32,
    pub dressed: u32,
    pub drops: u32,
    pub gate_x: f32,
    pub end_x: f32,
    pub flung: Vec<Vec3>,
    pub dead: bool,
    pub won: bool,
    pub restart: bool,
    pub stepped: bool,
}

#[derive(Default)]
pub struct Talk {
    pub line: String,
    pub shown: String,
    pub time: f32,
    pub queue: Vec<String>,
    pub next_taunt: u32,
    pub next_downed: u32,
    pub opened: bool,
}

#[derive(Default)]
pub struct Interface {
    pub caption: Entity,
    pub head_up: Entity,
    pub hero_fill: Entity,
    pub thug_panel: Entity,
    pub thug_fill: Entity,
    pub thug_name: Entity,
    pub score: Entity,
    pub stage: Entity,
    pub weapon: Entity,
    pub go: Entity,
    pub title: Entity,
    pub pause: Entity,
    pub over: Entity,
    pub over_title: Entity,
    pub over_score: Entity,
    pub story: Entity,
    pub arcade: Entity,
    pub leave: Entity,
    pub resume: Entity,
    pub menu: Entity,
    pub again: Entity,
    pub back: Entity,
    pub last_hero: f32,
    pub last_thug: f32,
    pub last_score: u32,
    pub ready: bool,
    pub shown_stage: String,
    pub shown_weapon: String,
    pub shown_name: &'static str,
    pub thug_shown: bool,
    pub chief_shown: bool,
    pub go_shown: bool,
}
