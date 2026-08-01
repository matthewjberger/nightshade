use nalgebra_glm::Mat4;
use nightshade::ecs::Entity;
use std::collections::HashMap;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSkinData {
    pub joint_count: u32,
    pub base_bone_index: u32,
    pub base_ibm_index: u32,
    pub base_output_index: u32,
}

pub fn skinned_root_bounds(global: &Mat4, rest_radius: f32) -> [f32; 4] {
    let scale = global
        .column(0)
        .xyz()
        .norm()
        .max(global.column(1).xyz().norm())
        .max(global.column(2).xyz().norm());
    [
        global[(0, 3)],
        global[(1, 3)],
        global[(2, 3)],
        rest_radius * scale * 2.0 + 0.5,
    ]
}

#[derive(Default, Clone)]
pub struct SkinningCache {
    pub inverse_bind_matrices: Vec<Mat4>,
    pub skin_data: Vec<GpuSkinData>,
    pub entity_skin_indices: HashMap<Entity, u32>,
    pub joint_entities: Vec<Entity>,
    pub total_joints: u32,
    pub skinned_entity_ids: Vec<u32>,
    pub static_data_uploaded: bool,
}

impl SkinningCache {
    pub fn get_joint_offset(&self, skin_index: u32) -> u32 {
        if let Some(skin_data) = self.skin_data.get(skin_index as usize) {
            skin_data.base_output_index
        } else {
            0
        }
    }

    pub fn get_base_bone_index(&self, skin_index: u32) -> u32 {
        if let Some(skin_data) = self.skin_data.get(skin_index as usize) {
            skin_data.base_bone_index
        } else {
            0
        }
    }
}
