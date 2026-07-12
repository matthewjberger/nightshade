//! Mesh geometry owned by the render layer: vertex formats, morph targets,
//! skin data, and the mesh container the caches and passes share.

use nalgebra_glm::Vec3;
use serde::{Deserialize, Serialize};

/// Standard vertex format for static meshes.
///
/// Layout matches the GPU vertex buffer format. Supports two UV sets,
/// tangent space for normal mapping, and per-vertex color.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
pub struct Vertex {
    /// Vertex position in local space.
    pub position: [f32; 3],
    /// Surface normal (should be normalized).
    pub normal: [f32; 3],
    /// Primary texture coordinates (UV0).
    pub tex_coords: [f32; 2],
    /// Secondary texture coordinates (UV1) for lightmaps or detail textures.
    pub tex_coords_1: [f32; 2],
    /// Tangent vector with handedness in W component (±1).
    pub tangent: [f32; 4],
    /// Per-vertex RGBA color.
    pub color: [f32; 4],
}

impl Vertex {
    /// Creates a vertex with position and normal, using default values for other attributes.
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

    /// Creates a vertex with position, normal, and primary texture coordinates.
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

    /// Creates a vertex with position, normal, texture coordinates, and tangent.
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

    /// Creates a vertex with all attributes explicitly specified.
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

/// Vertex format for skeletal animation with joint influences.
///
/// Extends [`Vertex`] with joint indices and weights for GPU skinning.
/// Each vertex can be influenced by up to 4 joints.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Serialize, Deserialize)]
pub struct SkinnedVertex {
    /// Vertex position in local space.
    pub position: [f32; 3],
    /// Surface normal (should be normalized).
    pub normal: [f32; 3],
    /// Primary texture coordinates (UV0).
    pub tex_coords: [f32; 2],
    /// Secondary texture coordinates (UV1).
    pub tex_coords_1: [f32; 2],
    /// Tangent vector with handedness in W component.
    pub tangent: [f32; 4],
    /// Per-vertex RGBA color.
    pub color: [f32; 4],
    /// Indices of influencing joints (up to 4).
    pub joint_indices: [u32; 4],
    /// Blend weights for each joint (should sum to 1.0).
    pub joint_weights: [f32; 4],
}

impl SkinnedVertex {
    /// Creates a skinned vertex with position, normal, UVs, and joint data.
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

    /// Creates a skinned vertex with all attributes explicitly specified.
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

/// A single morph target (blend shape) with vertex displacements.
///
/// Morph targets deform the base mesh by adding weighted displacements
/// to vertex positions, normals, and/or tangents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphTarget {
    /// Position offsets for each vertex.
    pub position_displacements: Vec<[f32; 3]>,
    /// Normal offsets for each vertex (optional).
    pub normal_displacements: Option<Vec<[f32; 3]>>,
    /// Tangent offsets for each vertex (optional).
    pub tangent_displacements: Option<Vec<[f32; 3]>>,
}

impl MorphTarget {
    /// Creates a morph target with position displacements only.
    pub fn new(position_displacements: Vec<[f32; 3]>) -> Self {
        Self {
            position_displacements,
            normal_displacements: None,
            tangent_displacements: None,
        }
    }

    /// Adds normal displacements to the morph target.
    pub fn with_normals(mut self, normal_displacements: Vec<[f32; 3]>) -> Self {
        self.normal_displacements = Some(normal_displacements);
        self
    }

    /// Adds tangent displacements to the morph target.
    pub fn with_tangents(mut self, tangent_displacements: Vec<[f32; 3]>) -> Self {
        self.tangent_displacements = Some(tangent_displacements);
        self
    }
}

/// Collection of morph targets for a mesh.
///
/// Contains all morph targets plus base mesh data needed for blending.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphTargetData {
    /// All morph targets for this mesh.
    pub targets: Vec<MorphTarget>,
    /// Default blend weights from the source asset.
    pub default_weights: Vec<f32>,
    /// Base mesh positions (before any morph deformation).
    pub base_positions: Vec<[f32; 3]>,
    /// Base mesh normals (before any morph deformation).
    pub base_normals: Vec<[f32; 3]>,
}

impl MorphTargetData {
    /// Creates morph target data with the given targets and default weights of 1.0.
    pub fn new(targets: Vec<MorphTarget>) -> Self {
        let default_weights = vec![1.0; targets.len()];
        Self {
            targets,
            default_weights,
            base_positions: Vec::new(),
            base_normals: Vec::new(),
        }
    }

    /// Sets the default blend weights for all targets.
    pub fn with_default_weights(mut self, weights: Vec<f32>) -> Self {
        self.default_weights = weights;
        self
    }

    /// Sets the base mesh positions and normals for CPU blending.
    pub fn with_base_data(mut self, positions: Vec<[f32; 3]>, normals: Vec<[f32; 3]>) -> Self {
        self.base_positions = positions;
        self.base_normals = normals;
        self
    }

    /// Returns the number of morph targets.
    pub fn target_count(&self) -> usize {
        self.targets.len()
    }
}

/// Skinning data for skeletal animation.
///
/// Contains the skinned vertex data and optionally references a skin definition
/// (joint hierarchy) by index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinData {
    /// Vertices with joint indices and weights.
    pub skinned_vertices: Vec<SkinnedVertex>,
    /// Index into the skin definitions array (from glTF or other source).
    pub skin_index: Option<usize>,
}

impl SkinData {
    /// Creates skin data with the given vertices.
    pub fn new(skinned_vertices: Vec<SkinnedVertex>) -> Self {
        Self {
            skinned_vertices,
            skin_index: None,
        }
    }

    /// Associates this skin data with a skin definition by index.
    pub fn with_skin_index(mut self, skin_index: usize) -> Self {
        self.skin_index = Some(skin_index);
        self
    }
}

/// CPU-side mesh data ready for GPU upload.
///
/// Contains geometry data (vertices and indices), optional bounding volume,
/// optional skinning data for skeletal animation, and optional morph targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    /// Vertex array with positions, normals, UVs, tangents, and colors.
    pub vertices: Vec<Vertex>,
    /// Triangle indices (3 per triangle).
    pub indices: Vec<u32>,
    /// Precomputed bounding volume for culling.
    pub bounding_volume: Option<crate::bounding_volume::BoundingVolume>,
    /// Skinning data for skeletal animation.
    pub skin_data: Option<SkinData>,
    /// Morph target data for blend shapes.
    pub morph_targets: Option<MorphTargetData>,
}

impl Mesh {
    /// Creates a mesh from vertices and indices.
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            bounding_volume: None,
            skin_data: None,
            morph_targets: None,
        }
    }

    /// Creates a mesh with a precomputed bounding volume.
    pub fn with_bounding_volume(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        bounding_volume: crate::bounding_volume::BoundingVolume,
    ) -> Self {
        Self {
            vertices,
            indices,
            bounding_volume: Some(bounding_volume),
            skin_data: None,
            morph_targets: None,
        }
    }

    /// Adds skinning data for skeletal animation.
    pub fn with_skin_data(mut self, skin_data: SkinData) -> Self {
        self.skin_data = Some(skin_data);
        self
    }

    /// Adds morph target data for blend shapes.
    pub fn with_morph_targets(mut self, morph_targets: MorphTargetData) -> Self {
        self.morph_targets = Some(morph_targets);
        self
    }

    /// Returns `true` if this mesh has skinning data.
    pub fn is_skinned(&self) -> bool {
        self.skin_data.is_some()
    }

    /// Returns `true` if this mesh has morph targets.
    pub fn has_morph_targets(&self) -> bool {
        self.morph_targets.is_some()
    }

    /// Returns the number of morph targets, or 0 if none.
    pub fn morph_target_count(&self) -> usize {
        self.morph_targets
            .as_ref()
            .map(|m| m.target_count())
            .unwrap_or(0)
    }
}

pub use crate::procedural_meshes::*;
