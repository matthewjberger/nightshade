mod instanced;
mod primitives;

pub use instanced::*;
pub use primitives::*;

use crate::ecs::asset_id::MeshId;
use nalgebra_glm::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMesh {
    pub name: String,
    #[serde(skip)]
    pub id: Option<MeshId>,
}

impl RenderMesh {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            id: None,
        }
    }

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

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub tangent: [f32; 4],
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            normal: [normal.x, normal.y, normal.z],
            tex_coords: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn with_tex_coords(position: Vec3, normal: Vec3, tex_coords: [f32; 2]) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            normal: [normal.x, normal.y, normal.z],
            tex_coords,
            tangent: [1.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn with_tangent(
        position: Vec3,
        normal: Vec3,
        tex_coords: [f32; 2],
        tangent: [f32; 4],
    ) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            normal: [normal.x, normal.y, normal.z],
            tex_coords,
            tangent,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
pub struct SkinnedVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub tangent: [f32; 4],
    pub joint_indices: [u32; 4],
    pub joint_weights: [f32; 4],
}

impl SkinnedVertex {
    pub fn new(
        position: Vec3,
        normal: Vec3,
        tex_coords: [f32; 2],
        joint_indices: [u32; 4],
        joint_weights: [f32; 4],
    ) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            normal: [normal.x, normal.y, normal.z],
            tex_coords,
            tangent: [1.0, 0.0, 0.0, 1.0],
            joint_indices,
            joint_weights,
        }
    }
}

impl Default for SkinnedVertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            joint_indices: [0, 0, 0, 0],
            joint_weights: [1.0, 0.0, 0.0, 0.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinData {
    pub skinned_vertices: Vec<SkinnedVertex>,
    pub skin_index: Option<usize>,
}

impl SkinData {
    pub fn new(skinned_vertices: Vec<SkinnedVertex>) -> Self {
        Self {
            skinned_vertices,
            skin_index: None,
        }
    }

    pub fn with_skin_index(mut self, skin_index: usize) -> Self {
        self.skin_index = Some(skin_index);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bounding_volume: Option<crate::ecs::bounding_volume::components::BoundingVolume>,
    pub skin_data: Option<SkinData>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            bounding_volume: None,
            skin_data: None,
        }
    }

    pub fn with_bounding_volume(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        bounding_volume: crate::ecs::bounding_volume::components::BoundingVolume,
    ) -> Self {
        Self {
            vertices,
            indices,
            bounding_volume: Some(bounding_volume),
            skin_data: None,
        }
    }

    pub fn with_skin_data(mut self, skin_data: SkinData) -> Self {
        self.skin_data = Some(skin_data);
        self
    }

    pub fn is_skinned(&self) -> bool {
        self.skin_data.is_some()
    }
}
