//! Skinning palette data owned by the render layer. The engine rebuilds and
//! refreshes it in `render_sync_skinning_system`; the skinned-mesh and shadow
//! passes read it from `renderer_state.render_skinning`.

use freecs::Entity;
use nalgebra_glm::Mat4;
use std::collections::HashMap;

/// Per-skin layout uploaded to the GPU: the joint count and the base offsets
/// into the shared bone, inverse-bind-matrix, and output-palette arrays.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSkinData {
    /// Number of joints in this skin.
    pub joint_count: u32,
    /// Offset of this skin's first joint in the bone matrix array.
    pub base_bone_index: u32,
    /// Offset of this skin's first inverse bind matrix.
    pub base_ibm_index: u32,
    /// Offset of this skin's first entry in the output palette.
    pub base_output_index: u32,
}

/// Conservative world-space bounding sphere for a skinned instance, derived
/// from its animated bone world positions plus a mesh-radius margin. Returns
/// `[center_x, center_y, center_z, radius]`. When the bone range is missing it
/// returns a never-cull sphere so a stale bound can only cost the cull, never
/// drop the instance.
pub fn skinned_world_bounds(
    bones: &[Mat4],
    base_bone: usize,
    joint_count: usize,
    margin: f32,
) -> [f32; 4] {
    if joint_count == 0 || base_bone + joint_count > bones.len() {
        return [0.0, 0.0, 0.0, 1.0e9];
    }
    let skin_bones = &bones[base_bone..base_bone + joint_count];
    let mut centroid = nalgebra_glm::vec3(0.0, 0.0, 0.0);
    for bone in skin_bones {
        centroid += nalgebra_glm::vec3(bone[(0, 3)], bone[(1, 3)], bone[(2, 3)]);
    }
    centroid /= joint_count as f32;
    let mut max_distance_squared = 0.0f32;
    for bone in skin_bones {
        let position = nalgebra_glm::vec3(bone[(0, 3)], bone[(1, 3)], bone[(2, 3)]);
        max_distance_squared = max_distance_squared.max((position - centroid).norm_squared());
    }
    let radius = max_distance_squared.sqrt() + margin;
    [centroid.x, centroid.y, centroid.z, radius]
}

/// Static skinning layout: deduplicated skins, their inverse bind matrices, and
/// the dense bone-index assignment. The engine populates it from Skin
/// components; the render passes only read it.
#[derive(Default, Clone)]
pub struct SkinningCache {
    /// Inverse bind matrices for every deduplicated skin, concatenated.
    pub inverse_bind_matrices: Vec<Mat4>,
    /// Per-skin layout records indexed by skin index.
    pub skin_data: Vec<GpuSkinData>,
    /// Skin index assigned to each skinned entity.
    pub entity_skin_indices: HashMap<Entity, u32>,
    /// Joint entities for every skin, concatenated in skin order.
    pub joint_entities: Vec<Entity>,
    /// Total joint count across all skins.
    pub total_joints: u32,
    /// Entity ids of the skinned instances in upload order.
    pub skinned_entity_ids: Vec<u32>,
    /// Whether the static per-skin data has been uploaded to the GPU.
    pub static_data_uploaded: bool,
}

impl SkinningCache {
    /// Offset of the skin's joint palette in the output array, or 0 if absent.
    pub fn get_joint_offset(&self, skin_index: u32) -> u32 {
        if let Some(skin_data) = self.skin_data.get(skin_index as usize) {
            skin_data.base_output_index
        } else {
            0
        }
    }

    /// Offset of the skin's first bone in the bone matrix array, or 0 if absent.
    pub fn get_base_bone_index(&self, skin_index: u32) -> u32 {
        if let Some(skin_data) = self.skin_data.get(skin_index as usize) {
            skin_data.base_bone_index
        } else {
            0
        }
    }
}
