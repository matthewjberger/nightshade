use nightshade::prelude::{Entity, Vec3};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Playing,
    Paused,
}

#[derive(Default)]
pub struct PortalsResources {
    pub player: PlayerState,
    pub hud: HudHandles,
    pub menu: MenuHandles,
    pub surfaces: [Entity; 2],
    pub solids: Vec<Solid>,
    pub last_rate: u32,
    pub last_held: bool,
}

pub struct PlayerState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub grabbed_now: bool,
    pub travel_hold: f32,
    pub camera: Entity,
    pub avatar: Entity,
    pub label: Entity,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            position: Vec3::new(
                crate::data::START[0],
                crate::data::START[1],
                crate::data::START[2],
            ),
            velocity: Vec3::zeros(),
            yaw: crate::data::START_YAW,
            pitch: 0.0,
            grabbed_now: false,
            travel_hold: 0.0,
            camera: Entity::default(),
            avatar: Entity::default(),
            label: Entity::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Solid {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Default, Clone, Copy)]
pub struct HudHandles {
    pub readout: Entity,
}

#[derive(Default, Clone, Copy)]
pub struct MenuHandles {
    pub resume: Entity,
    pub quit: Entity,
}
