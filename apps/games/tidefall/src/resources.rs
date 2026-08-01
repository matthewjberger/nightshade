use crate::data::{Drift, Globe, Marker, SCATTER_SEED};
use nightshade::ecs::Entity;
use nightshade_cli::clap;
use std::collections::HashSet;

pub use crate::data::mode::State as Mode;

#[derive(clap::Args, Default)]
pub struct Arguments {
    #[arg(long, help = "Start in the world rather than on the title screen")]
    pub playing: bool,
    #[arg(
        long,
        help = "Start in the cockpit rather than on foot beside the ship"
    )]
    pub aboard: bool,
    #[arg(long, help = "Lock this body as the target at the start")]
    pub target: Option<String>,
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
    pub settings_button: Entity,
    pub pause_title_button: Entity,
    pub settings_panel: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub root: Entity,
    pub canvas: Entity,
    pub mode_line: Entity,
    pub place_line: Entity,
    pub motion_line: Entity,
    pub target_panel: Entity,
    pub target_line: Entity,
    pub pilot_line: Entity,
    pub tally_line: Entity,
    pub near_line: Entity,
    pub talk_panel: Entity,
    pub talk_name: Entity,
    pub talk_text: Entity,
}

#[derive(Default)]
pub struct Readout {
    pub mode_line: String,
    pub place_line: String,
    pub motion_line: String,
    pub target_line: String,
    pub pilot_line: String,
    pub tally_line: String,
    pub near_line: String,
    pub talk_name: String,
    pub talk_text: String,
    pub talk_shown: bool,
    pub target_shown: bool,
    pub marker: Option<Marker>,
    pub globe: Option<Globe>,
    pub drift: Option<Drift>,
    pub gravity: Option<f32>,
    pub reticle: bool,
}

#[derive(Default)]
pub struct Shown {
    pub mode_line: String,
    pub place_line: String,
    pub motion_line: String,
    pub target_line: String,
    pub pilot_line: String,
    pub tally_line: String,
    pub near_line: String,
    pub talk_name: String,
    pub talk_text: String,
    pub talk_shown: Option<bool>,
    pub target_shown: Option<bool>,
    pub settings_open: bool,
}

pub struct Talk {
    pub name: String,
    pub lines: Vec<String>,
    pub at: usize,
}

pub struct TidefallResources {
    pub ship: Option<Entity>,
    pub walker: Option<Entity>,
    pub landings: u32,
    pub visited: HashSet<Entity>,
    pub found: HashSet<usize>,
    pub talk: Option<Talk>,
    pub headlights: Vec<Entity>,
    pub lights_on: bool,
    pub mode: Mode,
    pub roll: fastrand::Rng,
    pub hud: HudHandles,
    pub screens: ScreenHandles,
    pub readout: Readout,
    pub shown: Shown,
}

impl Default for TidefallResources {
    fn default() -> Self {
        Self {
            ship: None,
            walker: None,
            landings: 0,
            visited: HashSet::new(),
            found: HashSet::new(),
            talk: None,
            headlights: Vec::new(),
            lights_on: true,
            mode: Mode::default(),
            roll: fastrand::Rng::with_seed(SCATTER_SEED),
            hud: HudHandles::default(),
            screens: ScreenHandles::default(),
            readout: Readout::default(),
            shown: Shown::default(),
        }
    }
}
