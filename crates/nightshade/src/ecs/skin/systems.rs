use crate::ecs::world::{RENDER_MESH, SKIN, World};
use freecs::Entity;
use nalgebra_glm::Mat4;
use std::collections::HashMap;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSkinData {
    pub joint_count: u32,
    pub base_bone_index: u32,
    pub base_ibm_index: u32,
    pub base_output_index: u32,
}

pub struct SkinningUploadData {
    pub bone_transforms: Vec<Mat4>,
    pub inverse_bind_matrices: Vec<Mat4>,
    pub skin_data: Vec<GpuSkinData>,
    pub entity_skin_indices: HashMap<Entity, u32>,
    pub total_joints: u32,
}

impl SkinningUploadData {
    pub fn get_joint_offset(&self, skin_index: u32) -> u32 {
        if let Some(skin_data) = self.skin_data.get(skin_index as usize) {
            skin_data.base_output_index
        } else {
            0
        }
    }
}

#[derive(Default)]
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
    pub fn needs_rebuild(&self, world: &World) -> bool {
        if !self.static_data_uploaded {
            return true;
        }

        let current_entities: Vec<Entity> = world.query_entities(SKIN | RENDER_MESH).collect();

        if current_entities.len() != self.skinned_entity_ids.len() {
            return true;
        }

        for (index, entity) in current_entities.iter().enumerate() {
            if index >= self.skinned_entity_ids.len() || entity.id != self.skinned_entity_ids[index]
            {
                return true;
            }
        }

        false
    }

    pub fn rebuild_static_data(&mut self, world: &World) {
        let skinned_entities: Vec<Entity> = world.query_entities(SKIN | RENDER_MESH).collect();

        self.skinned_entity_ids = skinned_entities.iter().map(|e| e.id).collect();
        self.inverse_bind_matrices.clear();
        self.skin_data.clear();
        self.entity_skin_indices.clear();
        self.joint_entities.clear();
        self.total_joints = 0;

        if skinned_entities.is_empty() {
            self.static_data_uploaded = true;
            return;
        }

        let mut bone_entity_to_index: HashMap<Entity, u32> = HashMap::new();
        let mut bone_index = 0u32;

        for &entity in &skinned_entities {
            if let Some(skin) = world.get_skin(entity) {
                for &joint_entity in &skin.joints {
                    bone_entity_to_index.entry(joint_entity).or_insert_with(|| {
                        let index = bone_index;
                        self.joint_entities.push(joint_entity);
                        bone_index += 1;
                        index
                    });
                }
            }
        }

        let mut skin_configs: HashMap<u64, u32> = HashMap::new();
        let mut unique_skin_index = 0u32;
        let mut current_ibm_offset = 0u32;
        let mut current_output_offset = 0u32;

        for &entity in &skinned_entities {
            if let Some(skin) = world.get_skin(entity) {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                for joint in skin.joints.iter() {
                    use std::hash::Hash;
                    joint.id.hash(&mut hasher);
                }
                for matrix in skin.inverse_bind_matrices.iter() {
                    for row in 0..4 {
                        for column in 0..4 {
                            use std::hash::Hash;
                            matrix[(row, column)].to_bits().hash(&mut hasher);
                        }
                    }
                }
                use std::hash::Hasher;
                let skin_hash = hasher.finish();

                let skin_index = if let Some(&existing_index) = skin_configs.get(&skin_hash) {
                    existing_index
                } else {
                    let index = unique_skin_index;
                    skin_configs.insert(skin_hash, index);

                    let base_bone_index = if let Some(&first_joint) = skin.joints.first() {
                        bone_entity_to_index.get(&first_joint).copied().unwrap_or(0)
                    } else {
                        0
                    };

                    let joint_count = skin.joints.len().min(256) as u32;

                    let gpu_skin_data = GpuSkinData {
                        joint_count,
                        base_bone_index,
                        base_ibm_index: current_ibm_offset,
                        base_output_index: current_output_offset,
                    };
                    self.skin_data.push(gpu_skin_data);

                    for (joint_index, &inverse_bind) in
                        skin.inverse_bind_matrices.iter().enumerate()
                    {
                        if joint_index < 256 {
                            self.inverse_bind_matrices.push(inverse_bind);
                        }
                    }

                    current_ibm_offset += joint_count;
                    current_output_offset += joint_count;

                    unique_skin_index += 1;
                    index
                };

                self.entity_skin_indices.insert(entity, skin_index);
            }
        }

        self.total_joints = current_output_offset;
        self.static_data_uploaded = true;
    }

    pub fn collect_bone_transforms(&self, world: &World) -> Vec<Mat4> {
        let mut bone_transforms = Vec::with_capacity(self.joint_entities.len());

        for &joint_entity in &self.joint_entities {
            if let Some(joint_global_transform) = world.get_global_transform(joint_entity) {
                bone_transforms.push(joint_global_transform.0);
            } else {
                bone_transforms.push(Mat4::identity());
            }
        }

        bone_transforms
    }

    pub fn get_joint_offset(&self, skin_index: u32) -> u32 {
        if let Some(skin_data) = self.skin_data.get(skin_index as usize) {
            skin_data.base_output_index
        } else {
            0
        }
    }
}

pub fn collect_skinning_data(world: &World) -> SkinningUploadData {
    let skinned_entities: Vec<Entity> = world.query_entities(SKIN | RENDER_MESH).collect();

    if skinned_entities.is_empty() {
        return SkinningUploadData {
            bone_transforms: Vec::new(),
            inverse_bind_matrices: Vec::new(),
            skin_data: Vec::new(),
            entity_skin_indices: HashMap::new(),
            total_joints: 0,
        };
    }

    let mut bone_transforms = Vec::new();
    let mut inverse_bind_matrices = Vec::new();
    let mut skin_data_vec = Vec::new();
    let mut entity_skin_indices = HashMap::new();

    let mut bone_entity_to_index: HashMap<Entity, u32> = HashMap::new();
    let mut bone_index = 0u32;

    for &entity in &skinned_entities {
        if let Some(skin) = world.get_skin(entity) {
            for &joint_entity in &skin.joints {
                bone_entity_to_index.entry(joint_entity).or_insert_with(|| {
                    let index = bone_index;

                    if let Some(joint_global_transform) = world.get_global_transform(joint_entity) {
                        bone_transforms.push(joint_global_transform.0);
                    } else {
                        bone_transforms.push(Mat4::identity());
                    }

                    bone_index += 1;
                    index
                });
            }
        }
    }

    let mut skin_configs: HashMap<u64, u32> = HashMap::new();
    let mut unique_skin_index = 0u32;
    let mut current_ibm_offset = 0u32;
    let mut current_output_offset = 0u32;

    for &entity in &skinned_entities {
        if let Some(skin) = world.get_skin(entity) {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            for joint in skin.joints.iter() {
                use std::hash::Hash;
                joint.id.hash(&mut hasher);
            }
            for matrix in skin.inverse_bind_matrices.iter() {
                for row in 0..4 {
                    for column in 0..4 {
                        use std::hash::Hash;
                        matrix[(row, column)].to_bits().hash(&mut hasher);
                    }
                }
            }
            use std::hash::Hasher;
            let skin_hash = hasher.finish();

            let skin_index = if let Some(&existing_index) = skin_configs.get(&skin_hash) {
                existing_index
            } else {
                let index = unique_skin_index;
                skin_configs.insert(skin_hash, index);

                let base_bone_index = if let Some(&first_joint) = skin.joints.first() {
                    bone_entity_to_index.get(&first_joint).copied().unwrap_or(0)
                } else {
                    0
                };

                let joint_count = skin.joints.len().min(256) as u32;

                let gpu_skin_data = GpuSkinData {
                    joint_count,
                    base_bone_index,
                    base_ibm_index: current_ibm_offset,
                    base_output_index: current_output_offset,
                };
                skin_data_vec.push(gpu_skin_data);

                for (joint_index, &inverse_bind) in skin.inverse_bind_matrices.iter().enumerate() {
                    if joint_index < 256 {
                        inverse_bind_matrices.push(inverse_bind);
                    }
                }

                current_ibm_offset += joint_count;
                current_output_offset += joint_count;

                unique_skin_index += 1;
                index
            };

            entity_skin_indices.insert(entity, skin_index);
        }
    }

    SkinningUploadData {
        bone_transforms,
        inverse_bind_matrices,
        skin_data: skin_data_vec,
        entity_skin_indices,
        total_joints: current_output_offset,
    }
}
