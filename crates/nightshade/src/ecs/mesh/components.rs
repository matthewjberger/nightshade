//! Mesh component definitions.

mod instanced;

pub use instanced::*;

use crate::render::asset_id::MeshId;
use serde::{Deserialize, Serialize};

/// Component referencing a mesh by name in the mesh cache.
///
/// The `id` field is populated by the rendering system when the mesh is uploaded
/// to the GPU. The name is used to look up the mesh data in the cache.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMesh {
    /// Name of the mesh in the mesh cache.
    pub name: String,
    /// GPU mesh identifier, set by the renderer after upload.
    #[serde(skip)]
    pub id: Option<MeshId>,
}

impl RenderMesh {
    /// Creates a new mesh reference by name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            id: None,
        }
    }

    /// Creates a mesh reference with a pre-assigned GPU identifier.
    pub fn with_id(name: impl Into<String>, id: MeshId) -> Self {
        Self {
            name: name.into(),
            id: Some(id),
        }
    }
}

impl Default for RenderMesh {
    fn default() -> Self {
        Self {
            name: "Cube".to_string(),
            id: None,
        }
    }
}

impl From<String> for RenderMesh {
    fn from(name: String) -> Self {
        Self { name, id: None }
    }
}

impl From<&str> for RenderMesh {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            id: None,
        }
    }
}

/// Built-in geometry primitive types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GeometryPrimitive {
    Torus,
    Cube,
    Sphere,
    Plane,
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
            GeometryPrimitive::Cone => write!(f, "Cone"),
            GeometryPrimitive::Cylinder => write!(f, "Cylinder"),
        }
    }
}

pub use crate::render::geometry::{
    Mesh, MorphTarget, MorphTargetData, SkinData, SkinnedVertex, Vertex,
};
