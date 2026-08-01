use crate::data::notes::NOTE_ROWS;
use crate::data::rules::{MAX_HEALTH, NOCLIP_SPEED};
use nightshade::prelude::{Entity, Quat, Vec3, nalgebra_glm};
use nightshade_cli::clap;
use nightshade_load::LoadHandle;
use nightshade_prefab::Prefab;

#[derive(clap::Args, Default, Clone)]
pub struct Arguments {
    #[arg(long, help = "Start a run without the title screen")]
    pub straight_in: bool,
    #[arg(long, help = "Start with the shaft key already in hand")]
    pub with_key: bool,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Paused,
    Reading,
    Dead,
    Escaped,
}

#[derive(Clone, Copy)]
pub struct Console {
    pub fly_speed: f32,
    pub noclip_asked: bool,
    pub noclip: bool,
    pub invulnerable_asked: bool,
    pub invulnerable: bool,
}

impl Default for Console {
    fn default() -> Self {
        Self {
            fly_speed: NOCLIP_SPEED,
            noclip_asked: false,
            noclip: false,
            invulnerable_asked: false,
            invulnerable: false,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub root: Entity,
    pub prompt: Entity,
    pub message: Entity,
    pub cell: Entity,
    pub state: Entity,
    pub hurt: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct ScreenHandles {
    pub play_button: Entity,
    pub title_quit_button: Entity,
    pub resume_button: Entity,
    pub pause_title_button: Entity,
    pub dead_again_button: Entity,
    pub dead_title_button: Entity,
    pub escaped_title_button: Entity,
    pub note_heading: Entity,
    pub note_rows: [Entity; NOTE_ROWS],
    pub note_close_button: Entity,
}

pub struct PlayerState {
    pub body: Option<Entity>,
    pub eyes: Option<Entity>,
    pub torch: Option<Entity>,
    pub health: f32,
    pub head: Quat,
    pub at: Vec3,
    pub eye: Vec3,
    pub forward: Vec3,
    pub crouching: bool,
    pub moving: bool,
    pub sprinting: bool,
    pub footfall: f32,
    pub holding: Option<Entity>,
    pub hold_distance: f32,
    pub hold_offset: Vec3,
    pub hold_latched: bool,
    pub carrying_key: bool,
    pub spare_cells: u32,
    pub bite_cooldown: f32,
    pub noclip: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            body: None,
            eyes: None,
            torch: None,
            health: MAX_HEALTH,
            head: nalgebra_glm::quat_identity(),
            at: Vec3::zeros(),
            eye: Vec3::zeros(),
            forward: Vec3::new(0.0, 0.0, -1.0),
            crouching: false,
            moving: false,
            sprinting: false,
            footfall: 0.0,
            holding: None,
            hold_distance: 1.4,
            hold_offset: Vec3::zeros(),
            hold_latched: false,
            carrying_key: false,
            spare_cells: 0,
            bite_cooldown: 0.0,
            noclip: false,
        }
    }
}

#[derive(Default)]
pub struct RunState {
    pub clock: f32,
    pub message: String,
    pub message_until: f32,
    pub prompt: String,
    pub built: bool,
    pub hurt_until: f32,
    pub reading: Option<usize>,
}

#[derive(Default)]
pub struct ModelState {
    pub handles: Vec<LoadHandle>,
    pub prefabs: Vec<Option<Prefab>>,
    pub ready: bool,
    pub dressed: bool,
    pub wolf_handle: Option<LoadHandle>,
    pub wolf_spawned: bool,
    pub clip_prowl: Option<usize>,
    pub clip_charge: Option<usize>,
    pub clip_rest: Option<usize>,
}

#[derive(Default)]
pub struct BlackmarrowResources {
    pub screen: Screen,
    pub pending_screen: Option<Screen>,
    pub player: PlayerState,
    pub run: RunState,
    pub models: ModelState,
    pub hud: HudHandles,
    pub screens: ScreenHandles,
}
