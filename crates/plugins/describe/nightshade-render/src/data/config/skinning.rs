#[derive(Clone)]
pub struct SkinnedChannelData {
    pub key: u64,
    pub property: u32,
    pub interpolation: u32,
    pub input: std::sync::Arc<[f32]>,
    pub values: std::sync::Arc<[[f32; 4]]>,
    pub stride: u32,
}

#[derive(Clone)]
pub struct SkinnedJointData {
    pub local_index: u32,
    pub parent_local: Option<u32>,
    pub rest_translation: [f32; 3],
    pub rest_rotation: [f32; 4],
    pub rest_scale: [f32; 3],
    pub pose_channels: Vec<Vec<SkinnedChannelData>>,
    pub layer_channels: Vec<Vec<SkinnedChannelData>>,
    pub reference_channels: Vec<Vec<SkinnedChannelData>>,
}

#[derive(Clone)]
pub struct SkinnedIkChainData {
    pub chain_index: u32,
    pub root_local: u32,
    pub mid_local: u32,
    pub tip_local: u32,
    pub has_pole: bool,
}

#[derive(Clone)]
pub struct SkinnedMultiIkChainData {
    pub chain_index: u32,
    pub joint_locals: Vec<u32>,
}

#[derive(Clone)]
pub struct SkinnedAimConstraintData {
    pub chain_index: u32,
    pub joint_locals: Vec<u32>,
    pub forward: [f32; 3],
}

#[derive(Clone)]
pub struct SkinnedSpringChainData {
    pub chain_index: u32,
    pub joint_locals: Vec<u32>,
    pub rest_axes: Vec<[f32; 3]>,
    pub bone_lengths: Vec<f32>,
}

#[derive(Clone)]
pub struct SkinnedSkeletonData {
    pub skin_entity: nightshade::ecs::Entity,
    pub player_entity: nightshade::ecs::Entity,
    pub armature_parent: Option<nightshade::ecs::Entity>,
    pub joint_count: u32,
    pub active_count: u32,
    pub layer_count: u32,
    pub layer_modes: Vec<u32>,
    pub root_motion_local_index: Option<u32>,
    pub ik_chains: Vec<SkinnedIkChainData>,
    pub ik_multi_chains: Vec<SkinnedMultiIkChainData>,
    pub aim_constraints: Vec<SkinnedAimConstraintData>,
    pub spring_chains: Vec<SkinnedSpringChainData>,
    pub joints_ordered: Vec<SkinnedJointData>,
}

#[derive(Default, Clone)]
pub struct RenderSkinning {
    pub cache: crate::data::skinning::SkinningCache,
    pub bone_transforms: Vec<nalgebra_glm::Mat4>,
    pub static_generation: u64,
    pub bone_transforms_generation: u64,
}

#[derive(Clone, Default)]
pub struct SkinnedAnimationSnapshot {
    pub signature: u64,
    pub clip_signature: u64,
    pub skeletons: Vec<SkinnedSkeletonData>,
    pub pose_runtime: std::collections::HashMap<(nightshade::ecs::Entity, usize), [f32; 2]>,
    pub layer_runtime: std::collections::HashMap<(nightshade::ecs::Entity, usize), [f32; 2]>,
    pub ik_runtime: std::collections::HashMap<(nightshade::ecs::Entity, usize), [f32; 8]>,
    pub ik_multi_runtime: std::collections::HashMap<(nightshade::ecs::Entity, usize), [f32; 8]>,
    pub aim_runtime: std::collections::HashMap<(nightshade::ecs::Entity, usize), [f32; 8]>,
    pub spring_runtime: std::collections::HashMap<(nightshade::ecs::Entity, usize), [f32; 8]>,
    pub armature_roots: std::collections::HashMap<nightshade::ecs::Entity, nalgebra_glm::Mat4>,
}
