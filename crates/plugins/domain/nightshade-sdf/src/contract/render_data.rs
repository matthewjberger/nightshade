use nalgebra_glm::{Mat4, Vec3};
use nightshade_ecs::Entity;

use crate::contract::{SdfOperation, SdfShape};

#[derive(Clone, Debug, PartialEq)]
pub struct SdfEditInstance {
    pub entity: Entity,
    pub shape: SdfShape,
    pub operation: SdfOperation,
    pub blend: f32,
    pub tint: Vec3,
    pub roughness: f32,
    pub metal: f32,
    pub transform: Mat4,
    pub visible: bool,
    pub collides: bool,
}

#[derive(Default)]
pub struct SdfFrame {
    pub edits: Vec<SdfEditInstance>,
}

pub type SdfChunkKey = [i32; 3];

#[derive(Default, Clone, Debug)]
pub struct SdfCollisionMesh {
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

pub struct SdfClipmap {
    pub levels: u32,
    pub finest_cell: f32,
    pub bricks_per_side: u32,
}

impl Default for SdfClipmap {
    fn default() -> Self {
        Self {
            levels: 8,
            finest_cell: 0.05,
            bricks_per_side: 64,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfMarchScale(pub f32);

impl Default for SdfMarchScale {
    fn default() -> Self {
        Self(1.0)
    }
}

pub const SDF_BRICK_SAMPLES: u32 = 8;

pub const SDF_BRICK_CELLS: u32 = SDF_BRICK_SAMPLES - 1;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SdfDebugView {
    #[default]
    Lit,
    Bricks,
    Levels,
    Steps,
    Baked,
}

impl SdfDebugView {
    pub fn next(self) -> Self {
        match self {
            Self::Lit => Self::Bricks,
            Self::Bricks => Self::Levels,
            Self::Levels => Self::Steps,
            Self::Steps => Self::Baked,
            Self::Baked => Self::Lit,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Lit => "lit surface",
            Self::Bricks => "bricks",
            Self::Levels => "clipmap levels",
            Self::Steps => "march steps",
            Self::Baked => "baked bricks",
        }
    }
}
