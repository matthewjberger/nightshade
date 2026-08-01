use nalgebra_glm::Vec3;
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub tex_coords_1: [f32; 2],
    pub tangent: [f32; 4],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            normal: [normal.x, normal.y, normal.z],
            tex_coords: [0.0, 0.0],
            tex_coords_1: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn with_tex_coords(position: Vec3, normal: Vec3, tex_coords: [f32; 2]) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            normal: [normal.x, normal.y, normal.z],
            tex_coords,
            tex_coords_1: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
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
            tex_coords_1: [0.0, 0.0],
            tangent,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn with_all(
        position: Vec3,
        normal: Vec3,
        tex_coords: [f32; 2],
        tex_coords_1: [f32; 2],
        tangent: [f32; 4],
        color: [f32; 4],
    ) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            normal: [normal.x, normal.y, normal.z],
            tex_coords,
            tex_coords_1,
            tangent,
            color,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
pub struct SkinnedVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub tex_coords_1: [f32; 2],
    pub tangent: [f32; 4],
    pub color: [f32; 4],
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
            tex_coords_1: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            joint_indices,
            joint_weights,
        }
    }

    pub fn with_all(
        position: Vec3,
        normal: Vec3,
        tex_coords: [f32; 2],
        tex_coords_1: [f32; 2],
        tangent: [f32; 4],
        color: [f32; 4],
        joints: ([u32; 4], [f32; 4]),
    ) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            normal: [normal.x, normal.y, normal.z],
            tex_coords,
            tex_coords_1,
            tangent,
            color,
            joint_indices: joints.0,
            joint_weights: joints.1,
        }
    }
}

impl Default for SkinnedVertex {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            tex_coords_1: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            joint_indices: [0, 0, 0, 0],
            joint_weights: [1.0, 0.0, 0.0, 0.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphTarget {
    pub position_displacements: Vec<[f32; 3]>,
    pub normal_displacements: Option<Vec<[f32; 3]>>,
    pub tangent_displacements: Option<Vec<[f32; 3]>>,
}

impl MorphTarget {
    pub fn new(position_displacements: Vec<[f32; 3]>) -> Self {
        Self {
            position_displacements,
            normal_displacements: None,
            tangent_displacements: None,
        }
    }

    pub fn with_normals(mut self, normal_displacements: Vec<[f32; 3]>) -> Self {
        self.normal_displacements = Some(normal_displacements);
        self
    }

    pub fn with_tangents(mut self, tangent_displacements: Vec<[f32; 3]>) -> Self {
        self.tangent_displacements = Some(tangent_displacements);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphTargetData {
    pub targets: Vec<MorphTarget>,
    pub default_weights: Vec<f32>,
    pub base_positions: Vec<[f32; 3]>,
    pub base_normals: Vec<[f32; 3]>,
}

impl MorphTargetData {
    pub fn new(targets: Vec<MorphTarget>) -> Self {
        let default_weights = vec![1.0; targets.len()];
        Self {
            targets,
            default_weights,
            base_positions: Vec::new(),
            base_normals: Vec::new(),
        }
    }

    pub fn with_default_weights(mut self, weights: Vec<f32>) -> Self {
        self.default_weights = weights;
        self
    }

    pub fn with_base_data(mut self, positions: Vec<[f32; 3]>, normals: Vec<[f32; 3]>) -> Self {
        self.base_positions = positions;
        self.base_normals = normals;
        self
    }

    pub fn target_count(&self) -> usize {
        self.targets.len()
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
    pub bounding_volume: Option<crate::data::bounding_volume::BoundingVolume>,
    pub skin_data: Option<SkinData>,
    pub morph_targets: Option<MorphTargetData>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            bounding_volume: None,
            skin_data: None,
            morph_targets: None,
        }
    }

    pub fn with_bounding_volume(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        bounding_volume: crate::data::bounding_volume::BoundingVolume,
    ) -> Self {
        Self {
            vertices,
            indices,
            bounding_volume: Some(bounding_volume),
            skin_data: None,
            morph_targets: None,
        }
    }

    pub fn with_skin_data(mut self, skin_data: SkinData) -> Self {
        self.skin_data = Some(skin_data);
        self
    }

    pub fn with_morph_targets(mut self, morph_targets: MorphTargetData) -> Self {
        self.morph_targets = Some(morph_targets);
        self
    }

    pub fn is_skinned(&self) -> bool {
        self.skin_data.is_some()
    }

    pub fn has_morph_targets(&self) -> bool {
        self.morph_targets.is_some()
    }

    pub fn morph_target_count(&self) -> usize {
        self.morph_targets
            .as_ref()
            .map(|m| m.target_count())
            .unwrap_or(0)
    }
}

pub use crate::data::procedural_meshes::*;
