use bitflags::bitflags;
use freecs::Entity;
use nalgebra_glm::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    pub entity: Entity,
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

/// A penetration contact between a query shape and a collider.
/// `normal` points away from the collider, so moving the shape
/// center by `normal * depth` separates the pair. `point` is the
/// contact location on the query shape surface in world space.
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

#[cfg(feature = "physics")]
pub(crate) fn rigid_body_type_to_rapier(value: RigidBodyType) -> rapier3d::prelude::RigidBodyType {
    match value {
        RigidBodyType::Dynamic => rapier3d::prelude::RigidBodyType::Dynamic,
        RigidBodyType::Fixed => rapier3d::prelude::RigidBodyType::Fixed,
        RigidBodyType::KinematicPositionBased => {
            rapier3d::prelude::RigidBodyType::KinematicPositionBased
        }
        RigidBodyType::KinematicVelocityBased => {
            rapier3d::prelude::RigidBodyType::KinematicVelocityBased
        }
    }
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

#[cfg(feature = "physics")]
pub(crate) fn locked_axes_to_rapier(value: LockedAxes) -> rapier3d::prelude::LockedAxes {
    rapier3d::prelude::LockedAxes::from_bits_truncate(value.bits())
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

#[cfg(feature = "physics")]
pub(crate) fn interaction_groups_to_rapier(
    value: InteractionGroups,
) -> rapier3d::prelude::InteractionGroups {
    rapier3d::prelude::InteractionGroups::new(
        rapier3d::prelude::Group::from_bits_truncate(value.memberships),
        rapier3d::prelude::Group::from_bits_truncate(value.filter),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RigidBodyHandle(pub u64);

#[cfg(feature = "physics")]
pub(crate) fn rigid_body_handle_from_rapier(
    value: rapier3d::prelude::RigidBodyHandle,
) -> RigidBodyHandle {
    let (index, generation) = value.into_raw_parts();
    RigidBodyHandle(((generation as u64) << 32) | (index as u64))
}

#[cfg(feature = "physics")]
pub(crate) fn rigid_body_handle_to_rapier(
    value: RigidBodyHandle,
) -> rapier3d::prelude::RigidBodyHandle {
    let index = (value.0 & 0xFFFF_FFFF) as u32;
    let generation = ((value.0 >> 32) & 0xFFFF_FFFF) as u32;
    rapier3d::prelude::RigidBodyHandle::from_raw_parts(index, generation)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColliderHandle(pub u64);

#[cfg(feature = "physics")]
pub(crate) fn collider_handle_from_rapier(
    value: rapier3d::prelude::ColliderHandle,
) -> ColliderHandle {
    let (index, generation) = value.into_raw_parts();
    ColliderHandle(((generation as u64) << 32) | (index as u64))
}

#[cfg(feature = "physics")]
pub(crate) fn collider_handle_to_rapier(
    value: ColliderHandle,
) -> rapier3d::prelude::ColliderHandle {
    let index = (value.0 & 0xFFFF_FFFF) as u32;
    let generation = ((value.0 >> 32) & 0xFFFF_FFFF) as u32;
    rapier3d::prelude::ColliderHandle::from_raw_parts(index, generation)
}

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

#[cfg(feature = "physics")]
impl CharacterControllerConfig {
    pub(crate) fn to_rapier(&self, scale: f32) -> rapier3d::control::KinematicCharacterController {
        rapier3d::control::KinematicCharacterController {
            offset: rapier3d::control::CharacterLength::Absolute(self.offset * scale),
            up: nalgebra::Vector3::y_axis(),
            max_slope_climb_angle: self.max_slope_climb_angle,
            min_slope_slide_angle: self.min_slope_slide_angle,
            autostep: self.autostep_max_height.map(|max_height| {
                rapier3d::control::CharacterAutostep {
                    max_height: rapier3d::control::CharacterLength::Absolute(max_height * scale),
                    min_width: rapier3d::control::CharacterLength::Absolute(
                        self.autostep_min_width.unwrap_or(0.2) * scale,
                    ),
                    include_dynamic_bodies: self.autostep_include_dynamic_bodies,
                }
            }),
            snap_to_ground: self
                .snap_to_ground
                .map(|d| rapier3d::control::CharacterLength::Absolute(d * scale)),
            ..Default::default()
        }
    }
}
