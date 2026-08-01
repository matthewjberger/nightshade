use crate::contract::WorldSide;
use crate::data;
use nightshade::prelude::{Entity, Mat4, Quat, Vec3};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Playing,
    Paused,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Idle,
    Diving,
    Settling,
}

#[derive(Default)]
pub struct MashupResources {
    pub here: WorldSide,
    pub crossing: Crossing,
    pub meadow: MeadowState,
    pub quarry: QuarryState,
    pub frames: [Option<PortalFrame>; 2],
    pub camera: Entity,
    pub hud: HudHandles,
    pub menu: MenuHandles,
    pub grabbed_now: bool,
    pub last_note: String,
    pub view: Viewpoint,
}

#[derive(Clone, Copy)]
pub struct Viewpoint {
    pub eye: Vec3,
    pub look: Quat,
    pub field_of_view: f32,
}

impl Default for Viewpoint {
    fn default() -> Self {
        Self {
            eye: Vec3::zeros(),
            look: Quat::identity(),
            field_of_view: data::MEADOW_FIELD_OF_VIEW,
        }
    }
}

pub struct MeadowState {
    pub feet: Vec3,
    pub facing: f32,
    pub yaw: f32,
    pub yaw_index: i32,
    pub focus: Vec3,
    pub body: Entity,
    pub shadow: Entity,
}

impl Default for MeadowState {
    fn default() -> Self {
        Self {
            feet: Vec3::new(
                data::MEADOW_START[0],
                data::MEADOW_START[1],
                data::MEADOW_START[2],
            ),
            facing: 0.0,
            yaw: 0.0,
            yaw_index: 0,
            focus: Vec3::new(
                data::MEADOW_START[0],
                data::MEADOW_START[1],
                data::MEADOW_START[2],
            ),
            body: Entity::default(),
            shadow: Entity::default(),
        }
    }
}

pub struct QuarryState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub grounded: bool,
}

impl Default for QuarryState {
    fn default() -> Self {
        Self {
            position: Vec3::new(
                data::QUARRY_START[0],
                data::QUARRY_START[1],
                data::QUARRY_START[2],
            ),
            velocity: Vec3::zeros(),
            yaw: 0.0,
            pitch: 0.0,
            grounded: false,
        }
    }
}

#[derive(Default)]
pub struct Crossing {
    pub phase: Phase,
    pub elapsed: f32,
    pub hold: f32,
    pub from_eye: Vec3,
    pub from_look: Quat,
    pub to_eye: Vec3,
    pub to_look: Quat,
    pub toward: WorldSide,
    pub settle_time: f32,
}

#[derive(Clone, Copy)]
pub struct PortalFrame {
    pub model: Mat4,
    pub eye: Entity,
    pub rim: [f32; 4],
    pub fade: f32,
}

#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub readout: Entity,
    pub crosshair: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct MenuHandles {
    pub resume: Entity,
    pub quit: Entity,
}
