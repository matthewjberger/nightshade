pub mod instanced;

pub use instanced::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMesh {
    pub name: String,
}

impl RenderMesh {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Default for RenderMesh {
    fn default() -> Self {
        Self {
            name: "Cube".to_string(),
        }
    }
}

impl From<String> for RenderMesh {
    fn from(name: String) -> Self {
        Self { name }
    }
}

impl From<&str> for RenderMesh {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugViewExempt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GeometryPrimitive {
    Torus,
    Cube,
    Sphere,
    Plane,
    Quad,
    Cone,
    Cylinder,
}

impl std::fmt::Display for GeometryPrimitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeometryPrimitive::Torus => write!(f, "Torus"),
            GeometryPrimitive::Cube => write!(f, "Cube"),
            GeometryPrimitive::Sphere => write!(f, "Sphere"),
            GeometryPrimitive::Plane => write!(f, "Plane"),
            GeometryPrimitive::Quad => write!(f, "Quad"),
            GeometryPrimitive::Cone => write!(f, "Cone"),
            GeometryPrimitive::Cylinder => write!(f, "Cylinder"),
        }
    }
}

pub use crate::data::geometry::{
    Mesh, MorphTarget, MorphTargetData, SkinData, SkinnedVertex, Vertex,
};
