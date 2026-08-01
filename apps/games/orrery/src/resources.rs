use nightshade::prelude::{Entity, Quat, Vec3};
use nightshade_cli::clap;

#[derive(clap::Args, Default, Clone)]
pub struct Arguments {
    #[arg(long, help = "Open in flight instead of on the title screen")]
    pub fly: bool,
    #[arg(long, help = "Park the ship beside the named body")]
    pub at: Option<String>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Title,
    Playing,
    Paused,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Flying,
    OnFoot,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum View {
    #[default]
    Chase,
    Cockpit,
}

#[derive(Clone, Copy)]
pub struct Signal {
    pub body: usize,
    pub distance: f32,
}

#[derive(Default, Clone, Copy)]
pub struct SceneHandles {
    pub ship: Entity,
    pub plume: Entity,
    pub walker: Entity,
    pub camera: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub root: Entity,
    pub readout: Entity,
    pub prompt: Entity,
    pub title_panel: Entity,
    pub fly_button: Entity,
    pub title_exit_button: Entity,
    pub pause_panel: Entity,
    pub resume_button: Entity,
    pub title_return_button: Entity,
    pub pause_exit_button: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct Look {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Clone, Copy)]
pub struct Pose {
    pub ship_position: Vec3,
    pub ship_rotation: Quat,
    pub ship_velocity: Vec3,
    pub thrust: Vec3,
    pub walker_position: Vec3,
    pub walker_up: Vec3,
    pub landed: Option<usize>,
    pub target: usize,
    pub autopilot: bool,
    pub nearest: usize,
    pub height: f32,
    pub relative_speed: f32,
    pub target_distance: f32,
}

impl Default for Pose {
    fn default() -> Self {
        Self {
            ship_position: Vec3::zeros(),
            ship_rotation: Quat::identity(),
            ship_velocity: Vec3::zeros(),
            thrust: Vec3::zeros(),
            walker_position: Vec3::zeros(),
            walker_up: Vec3::y(),
            landed: None,
            target: 0,
            autopilot: false,
            nearest: 0,
            height: 0.0,
            relative_speed: 0.0,
            target_distance: 0.0,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Flight {
    pub mode: Mode,
    pub view: View,
    pub elapsed: f32,
    pub found: u32,
    pub total: u32,
    pub knocks: u32,
}

#[derive(Default)]
pub struct OrreryResources {
    pub scene: SceneHandles,
    pub hud: HudHandles,
    pub look: Look,
    pub pose: Pose,
    pub flight: Flight,
    pub screen: Screen,
    pub shown: Option<Screen>,
    pub wanted: Option<Screen>,
    pub cursor_applied: Option<bool>,
    pub sweep: Option<Signal>,
    pub log: Vec<Entity>,
    pub logged: Vec<usize>,
    pub readout: String,
    pub prompt: String,
}
