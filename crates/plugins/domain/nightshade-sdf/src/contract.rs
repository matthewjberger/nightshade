pub mod field;
pub mod render_data;

use nalgebra_glm::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, enum2schema::Schema)]
pub enum SdfShape {
    Sphere {
        radius: f32,
    },
    Cube {
        half_extents: Vec3,
        rounding: f32,
    },
    Capsule {
        half_height: f32,
        radius: f32,
    },
    Torus {
        major_radius: f32,
        minor_radius: f32,
    },
    Cylinder {
        half_height: f32,
        radius: f32,
    },
    Octahedron {
        radius: f32,
        rounding: f32,
    },
    Terrain {
        base: f32,
        size: f32,
        octaves: u32,
        gain: f32,
        clip: f32,
        clip_smooth: f32,
        join_smooth: f32,
        skin: f32,
        dirt: Vec3,
    },
}

impl Default for SdfShape {
    fn default() -> Self {
        Self::Sphere { radius: 1.0 }
    }
}

impl SdfShape {
    pub fn reach(&self) -> Option<f32> {
        match *self {
            Self::Sphere { radius } => Some(radius),
            Self::Cube {
                half_extents,
                rounding,
            } => Some(half_extents.magnitude() + rounding),
            Self::Capsule {
                half_height,
                radius,
            } => Some(half_height + radius),
            Self::Torus {
                major_radius,
                minor_radius,
            } => Some(major_radius + minor_radius),
            Self::Cylinder {
                half_height,
                radius,
            } => Some((half_height * half_height + radius * radius).sqrt()),
            Self::Octahedron { radius, rounding } => Some(radius + rounding),
            Self::Terrain { .. } => None,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, enum2schema::Schema,
)]
#[schema(string_enum)]
pub enum SdfOperation {
    #[default]
    Union,
    Subtract,
    Intersect,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SdfBlend(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SdfTint(pub Vec3);

impl Default for SdfTint {
    fn default() -> Self {
        Self(Vec3::new(0.72, 0.70, 0.66))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct SdfOrder(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SdfRoughness(pub f32);

impl Default for SdfRoughness {
    fn default() -> Self {
        Self(0.8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SdfMetal(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct SdfCollisionFocus(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SdfCollides(pub bool);

impl Default for SdfCollides {
    fn default() -> Self {
        Self(true)
    }
}
