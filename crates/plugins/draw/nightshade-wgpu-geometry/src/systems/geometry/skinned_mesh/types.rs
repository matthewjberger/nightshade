pub(super) use nightshade_wgpu_shading::backend::projection::LightData;
pub(super) use nightshade_wgpu_shading::backend::scene_uniforms::SceneUniforms;

pub use nightshade_wgpu_shading::backend::cluster::{
    CLUSTER_GRID_X, CLUSTER_GRID_Y, CLUSTER_GRID_Z, ClusterBounds, ClusterUniforms, LightGrid,
    MAX_LIGHTS_PER_CLUSTER, TOTAL_CLUSTERS, cluster_slice_near,
};
pub(super) use nightshade_wgpu_shading::backend::projection::{
    AreaLightData, AreaUniforms, MAX_AREA_LIGHTS,
};

pub(super) const MAX_JOINTS_PER_SKIN: usize = 256;
pub(super) const INITIAL_MAX_SKINS: usize = 100;
pub(super) const INITIAL_MAX_TOTAL_BONES: usize = 5_000;
pub(super) const INITIAL_SKINNED_VERTEX_COUNT: usize = 100_000;
pub(super) const INITIAL_SKINNED_INDEX_COUNT: usize = 300_000;
pub(super) const INITIAL_MORPH_DISPLACEMENT_COUNT: usize = 100_000;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct SkinnedObjectData {
    pub(super) transform_index: u32,
    pub(super) mesh_id: u32,
    pub(super) material_id: u32,
    pub(super) joint_offset: u32,
    pub(super) morph_weights: [f32; 8],
    pub(super) morph_target_count: u32,
    pub(super) morph_displacement_offset: u32,
    pub(super) mesh_vertex_offset: u32,
    pub(super) mesh_vertex_count: u32,
    pub(super) entity_id: u32,
    pub(super) is_transparent: u32,
    pub(super) flip_winding: u32,
    pub(super) debug_view_exempt: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct MorphDisplacement {
    pub(super) position: [f32; 3],
    pub(super) padding_0: f32,
    pub(super) normal: [f32; 3],
    pub(super) padding_1: f32,
    pub(super) tangent: [f32; 3],
    pub(super) padding_2: f32,
}

pub use nightshade_wgpu_shading::backend::material_gpu::MaterialData;

#[derive(Debug, Clone)]
pub(super) struct SkinnedMeshData {
    pub(super) vertex_offset: u32,
    pub(super) vertex_count: u32,
    pub(super) index_offset: u32,
    pub(super) index_count: u32,
    pub(super) morph_displacement_offset: u32,
    pub(super) morph_target_count: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct SkinnedCullObject {
    pub(super) bounds: [f32; 4],
    pub(super) command_index: u32,
    pub(super) padding: [u32; 3],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct SkinnedBoundsInput {
    pub(super) base_bone: u32,
    pub(super) joint_count: u32,
    pub(super) rest_radius: f32,
    pub(super) pad: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct SkinnedDrawIndexedIndirect {
    pub(super) index_count: u32,
    pub(super) instance_count: u32,
    pub(super) first_index: u32,
    pub(super) base_vertex: i32,
    pub(super) first_instance: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct SkinnedCullUniforms {
    pub(super) frustum_planes: [[f32; 4]; 6],
    pub(super) object_count: u32,
    pub(super) padding: [u32; 3],
}
