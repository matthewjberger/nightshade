use bitflags::bitflags;
use nalgebra_glm;
use nalgebra_glm::Vec3;
use nightshade_ecs;
use nightshade_ecs::Entity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    pub entity: Entity,
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct ShapeContact {
    pub entity: Entity,
    pub point: Vec3,
    pub normal: Vec3,
    pub depth: f32,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, enum2schema::Schema,
)]
#[schema(string_enum)]
pub enum RigidBodyType {
    #[default]
    Dynamic,
    Fixed,
    KinematicPositionBased,
    KinematicVelocityBased,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
    pub struct LockedAxes: u8 {
        const TRANSLATION_LOCKED_X = 0b0000_0001;
        const TRANSLATION_LOCKED_Y = 0b0000_0010;
        const TRANSLATION_LOCKED_Z = 0b0000_0100;
        const ROTATION_LOCKED_X = 0b0000_1000;
        const ROTATION_LOCKED_Y = 0b0001_0000;
        const ROTATION_LOCKED_Z = 0b0010_0000;
        const TRANSLATION_LOCKED = Self::TRANSLATION_LOCKED_X.bits() | Self::TRANSLATION_LOCKED_Y.bits() | Self::TRANSLATION_LOCKED_Z.bits();
        const ROTATION_LOCKED = Self::ROTATION_LOCKED_X.bits() | Self::ROTATION_LOCKED_Y.bits() | Self::ROTATION_LOCKED_Z.bits();
    }
}

impl enum2schema::Schema for LockedAxes {
    fn schema() -> enum2schema::serde_json::Value {
        enum2schema::serde_json::json!({
            "type": "integer",
            "minimum": 0,
            "maximum": 63
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, enum2schema::Schema)]
pub struct InteractionGroups {
    pub memberships: u32,
    pub filter: u32,
}

impl Default for InteractionGroups {
    fn default() -> Self {
        Self::all()
    }
}

impl InteractionGroups {
    pub fn new(memberships: u32, filter: u32) -> Self {
        Self {
            memberships,
            filter,
        }
    }

    pub fn all() -> Self {
        Self {
            memberships: u32::MAX,
            filter: u32::MAX,
        }
    }

    pub fn none() -> Self {
        Self {
            memberships: 0,
            filter: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RigidBodyHandle(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColliderHandle(pub u64);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
pub struct CharacterControllerConfig {
    pub offset: f32,
    pub max_slope_climb_angle: f32,
    pub min_slope_slide_angle: f32,
    pub autostep_max_height: Option<f32>,
    pub autostep_min_width: Option<f32>,
    pub autostep_include_dynamic_bodies: bool,
    pub snap_to_ground: Option<f32>,
}

impl Default for CharacterControllerConfig {
    fn default() -> Self {
        Self {
            offset: 0.01,
            max_slope_climb_angle: 45_f32.to_radians(),
            min_slope_slide_angle: 30_f32.to_radians(),
            autostep_max_height: Some(0.3),
            autostep_min_width: Some(0.2),
            autostep_include_dynamic_bodies: false,
            snap_to_ground: Some(0.2),
        }
    }
}
impl CharacterControllerConfig {}
