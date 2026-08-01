use crate::data::{self, Ground};
use fastrand::Rng;
use nalgebra_glm::Vec2;
use nightshade::prelude::Entity;
use nightshade_cli::clap;
use nightshade_sprite::prelude::{SpriteRegion, TileTrim};

#[derive(clap::Args, Default)]
pub struct Arguments {
    #[arg(long, value_name = "HOUR", help = "Start the day at this hour")]
    pub hour: Option<f32>,
    #[arg(long, value_name = "COLUMN", help = "Start at this ring column")]
    pub column: Option<f32>,
    #[arg(long, help = "Open the window over the whole screen")]
    pub fullscreen: bool,
    #[arg(long, help = "Walk into the village without the title screen")]
    pub play: bool,
}

pub struct Town {
    pub columns: u32,
    pub rows: u32,
    pub heights: Vec<u32>,
    pub ground: Vec<Ground>,
    pub blocked: Vec<bool>,
}

impl Default for Town {
    fn default() -> Self {
        Self {
            columns: data::ACROSS,
            rows: data::AROUND,
            heights: Vec::new(),
            ground: Vec::new(),
            blocked: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct Village {
    pub player: Option<Entity>,
    pub camera: Option<Entity>,
    pub sun: Option<Entity>,
    pub turntable: Option<Entity>,
    pub feet: Vec2,
    pub lift: f32,
}

#[derive(Default)]
pub struct Rig {
    pub focus: Vec2,
    pub lift: f32,
}

#[derive(Clone, Copy)]
pub struct Sheet {
    pub tiles: SpriteRegion,
    pub shadow: SpriteRegion,
    pub trim: [TileTrim; data::SHEET_TILES],
    pub villagers: [SpriteRegion; data::NEIGHBORS.len()],
    pub shell: SpriteRegion,
    pub fish: SpriteRegion,
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            tiles: SpriteRegion::default(),
            shadow: SpriteRegion::default(),
            trim: [TileTrim::default(); data::SHEET_TILES],
            villagers: [SpriteRegion::default(); data::NEIGHBORS.len()],
            shell: SpriteRegion::default(),
            fish: SpriteRegion::default(),
        }
    }
}

pub struct Sky {
    pub hour: f32,
    pub tint: [f32; 3],
    pub breeze: f32,
}

impl Default for Sky {
    fn default() -> Self {
        Self {
            hour: data::HOUR,
            tint: [0.0, 0.0, 0.0],
            breeze: 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum WeatherKind {
    #[default]
    Clear,
    Cloudy,
    Rain,
}

pub struct Weather {
    pub kind: WeatherKind,
    pub until: f32,
    pub cloud: f32,
    pub rain: f32,
    pub emitter: Option<Entity>,
    pub rng: Rng,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            kind: WeatherKind::Clear,
            until: data::WEATHER_SPELL,
            cloud: 0.0,
            rain: 0.0,
            emitter: None,
            rng: Rng::with_seed(8123_u64),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Talking,
    Shop,
    Paused,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Deed {
    #[default]
    None,
    Talk(usize),
    Pick(Entity),
    Shake(Entity),
    Knock(Entity),
    Cast,
    Reel,
    Shop(usize),
}

#[derive(Default)]
pub struct Prompt {
    pub deed: Deed,
    pub line: String,
}

pub struct Pocket {
    pub counts: [u32; data::Item::ALL.len()],
    pub bells: u32,
}

impl Default for Pocket {
    fn default() -> Self {
        Self {
            counts: [0; data::Item::ALL.len()],
            bells: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FishingState {
    #[default]
    Idle,
    Waiting,
    Bite,
}

#[derive(Default)]
pub struct Fishing {
    pub state: FishingState,
    pub timer: f32,
    pub bobber: Option<Entity>,
    pub rng: Rng,
}

#[derive(Default)]
pub struct Beach {
    pub timer: f32,
    pub rng: Rng,
}

#[derive(Default)]
pub struct Toast {
    pub line: String,
    pub remaining: f32,
}

#[derive(Default)]
pub struct Store {
    pub keeper: Option<usize>,
    pub sold: u32,
}

#[derive(Default)]
pub struct Chat {
    pub villager: Option<usize>,
    pub line: usize,
    pub shown: bool,
}

#[derive(Default, Clone, Copy)]
pub struct Hud {
    pub ready: bool,
    pub root: Entity,
    pub place_panel: Entity,
    pub pocket_panel: Entity,
    pub place_text: Entity,
    pub clock_text: Entity,
    pub toast_text: Entity,
    pub bells_text: Entity,
    pub pocket_rows: [Entity; data::Item::ALL.len()],
    pub prompt_panel: Entity,
    pub prompt_text: Entity,
    pub speech_panel: Entity,
    pub speaker_text: Entity,
    pub speech_text: Entity,
    pub shop_panel: Entity,
    pub shop_title: Entity,
    pub shop_rows: [Entity; data::Item::ALL.len()],
    pub shop_total: Entity,
    pub sell_button: Entity,
    pub leave_button: Entity,
    pub veil: Entity,
    pub title_panel: Entity,
    pub start_button: Entity,
    pub title_quit_button: Entity,
    pub pause_panel: Entity,
    pub resume_button: Entity,
    pub quit_button: Entity,
}

#[derive(Default)]
pub struct Readouts {
    pub place: String,
    pub clock: String,
    pub toast: String,
    pub bells: String,
    pub pocket: [String; data::Item::ALL.len()],
    pub prompt: String,
    pub speaker: String,
    pub speech: String,
    pub shop: [String; data::Item::ALL.len()],
    pub shop_title: String,
    pub shop_total: String,
    pub prompt_open: bool,
    pub speech_open: bool,
    pub shop_open: bool,
    pub title_open: bool,
    pub pause_open: bool,
    pub veil_open: bool,
    pub hud_open: bool,
    pub cursor_free: bool,
}
