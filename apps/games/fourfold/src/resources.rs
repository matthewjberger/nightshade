use crate::contract::Glyph;
use crate::data;
use nightshade::prelude::{Entity, Vec2, Vec3};
use nightshade_cli::clap;

#[derive(clap::Args, Default, Clone)]
pub struct Arguments {
    #[arg(long, help = "Let the game play itself from the title")]
    pub autoplay: bool,
}

#[derive(Default, Clone, Copy)]
pub struct Drive {
    pub wish: Vec2,
    pub look: Vec2,
    pub jump: bool,
    pub dial: Option<usize>,
}

pub struct Strider {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub grounded: bool,
    pub focus: Vec3,
}

impl Default for Strider {
    fn default() -> Self {
        Self {
            position: data::HEARTH_START,
            velocity: Vec3::zeros(),
            yaw: 0.0,
            grounded: false,
            focus: data::HEARTH_START,
        }
    }
}

pub struct Stepper {
    pub cell: [i32; 2],
    pub rest: f32,
}

impl Default for Stepper {
    fn default() -> Self {
        Self {
            cell: data::LOOM_START,
            rest: 0.0,
        }
    }
}

#[derive(Default)]
pub struct Autoplay {
    pub on: bool,
    pub step: usize,
    pub held: f32,
    pub jumps: usize,
    pub moves: usize,
}

pub struct FourfoldResources {
    pub walker: Walker,
    pub runner: Runner,
    pub stepper: Stepper,
    pub digger: Walker,
    pub villager: Strider,
    pub figure: Entity,
    pub cairn_camera: Entity,
    pub hearth_camera: Entity,
    pub lit: [bool; data::DIAL_COUNT],
    pub frames: [Entity; data::DIAL_COUNT],
    pub pawn: Entity,
    pub loom_camera: Entity,
    pub arch: ArchState,
    pub hud: HudHandles,
    pub menu: MenuButtons,
    pub hub_camera: Entity,
    pub glass_camera: Entity,
    pub seal: Entity,
    pub avatar: Entity,
    pub crossings: u32,
    pub story: Story,
    pub drive: Drive,
    pub autoplay: Autoplay,
    pub fade: f32,
    pub ending: f32,
    pub last_note: String,
    pub last_prompt: String,
    pub crosshair_shown: bool,
}

impl Default for FourfoldResources {
    fn default() -> Self {
        Self {
            walker: Walker::default(),
            runner: Runner::default(),
            stepper: Stepper::default(),
            digger: Walker {
                position: data::CAIRN_START,
                ..Default::default()
            },
            villager: Strider::default(),
            figure: Entity::default(),
            cairn_camera: Entity::default(),
            hearth_camera: Entity::default(),
            lit: [false; data::DIAL_COUNT],
            frames: [Entity::default(); data::DIAL_COUNT],
            pawn: Entity::default(),
            loom_camera: Entity::default(),
            arch: ArchState::default(),
            hud: HudHandles::default(),
            menu: MenuButtons::default(),
            hub_camera: Entity::default(),
            glass_camera: Entity::default(),
            seal: Entity::default(),
            avatar: Entity::default(),
            crossings: 0,
            story: Story::default(),
            drive: Drive::default(),
            autoplay: Autoplay::default(),
            fade: 0.0,
            ending: 0.0,
            last_note: String::new(),
            last_prompt: String::new(),
            crosshair_shown: true,
        }
    }
}

pub struct Walker {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub grounded: bool,
    pub since_ground: f32,
    pub near_dials: bool,
}

impl Default for Walker {
    fn default() -> Self {
        Self {
            position: data::HUB_START,
            velocity: Vec3::zeros(),
            yaw: 0.0,
            pitch: 0.0,
            grounded: false,
            since_ground: 0.0,
            near_dials: false,
        }
    }
}

pub struct Runner {
    pub position: Vec3,
    pub velocity: Vec3,
    pub grounded: bool,
    pub since_ground: f32,
    pub armed: bool,
    pub focus: Vec3,
}

impl Default for Runner {
    fn default() -> Self {
        Self {
            position: data::GLASS_START,
            velocity: Vec3::zeros(),
            grounded: false,
            since_ground: 0.0,
            armed: false,
            focus: data::GLASS_START,
        }
    }
}

pub struct ArchState {
    pub dials: [Glyph; data::DIAL_COUNT],
    pub open: bool,
}

impl Default for ArchState {
    fn default() -> Self {
        Self {
            dials: [Glyph::Bar; data::DIAL_COUNT],
            open: false,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Paused,
}

#[derive(Default, Clone, Copy)]
pub struct MenuButtons {
    pub play: Entity,
    pub title_quit: Entity,
    pub resume: Entity,
    pub to_title: Entity,
    pub pause_quit: Entity,
}

#[derive(Default)]
pub struct Story {
    pub fired: [bool; data::SCRIPT_COUNT],
    pub queue: Vec<usize>,
    pub showing: Option<usize>,
    pub painted: Option<usize>,
    pub held: f32,
}

#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub root: Entity,
    pub readout: Entity,
    pub prompt: Entity,
    pub crosshair: Entity,
    pub speaker: Entity,
    pub line: Entity,
    pub fade: Entity,
}
