pub(super) use super::super::projection::LightData;

pub struct BlendedIblViews {
    pub brdf_lut: wgpu::TextureView,
    pub irradiance_a: wgpu::TextureView,
    pub prefiltered_a: wgpu::TextureView,
    pub irradiance_b: wgpu::TextureView,
    pub prefiltered_b: wgpu::TextureView,
    pub blend_factor: f32,
}

pub use super::super::cluster::{
    CLUSTER_GRID_X, CLUSTER_GRID_Y, CLUSTER_GRID_Z, ClusterBounds, ClusterUniforms, LightGrid,
    MAX_LIGHTS_PER_CLUSTER, TOTAL_CLUSTERS, cluster_slice_near,
};
pub(super) use super::super::projection::{AreaLightData, AreaUniforms, MAX_AREA_LIGHTS};

pub(super) const MAX_JOINTS_PER_SKIN: usize = 256;
pub(super) const INITIAL_MAX_SKINS: usize = 100;
pub(super) const INITIAL_MAX_TOTAL_BONES: usize = 5_000;
pub(super) const INITIAL_SKINNED_VERTEX_COUNT: usize = 100_000;
pub(super) const INITIAL_SKINNED_INDEX_COUNT: usize = 300_000;
pub(super) const INITIAL_MORPH_DISPLACEMENT_COUNT: usize = 100_000;

#[repr(C, align(256))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct SkinnedMeshUniforms {
    pub(super) view: [[f32; 4]; 4],
    pub(super) projection: [[f32; 4]; 4],
    pub(super) camera_position: [f32; 4],
    pub(super) num_lights: [u32; 4],
    pub(super) ambient_light: [f32; 4],
    pub(super) light_view_projection: [[f32; 4]; 4],
    pub(super) shadow_bias: f32,
    pub(super) shadows_enabled: f32,
    pub(super) global_unlit: f32,
    pub(super) shadow_normal_bias: f32,
    pub(super) snap_resolution: [f32; 2],
    pub(super) snap_enabled: u32,
    pub(super) affine_enabled: u32,
    pub(super) fog_color: [f32; 3],
    pub(super) fog_enabled: u32,
    pub(super) fog_start: f32,
    pub(super) fog_end: f32,
    pub(super) cascade_count: u32,
    pub(super) _padding2: f32,
    pub(super) cascade_view_projections: [[[f32; 4]; 4]; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    pub(super) cascade_split_distances: [f32; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    pub(super) cascade_atlas_offsets: [[f32; 4]; crate::wgpu::passes::NUM_SHADOW_CASCADES],
    pub(super) cascade_atlas_scale: [f32; 4],
    pub(super) time: f32,
    pub(super) pbr_debug_mode: u32,
    pub(super) texture_debug_stripes: u32,
    pub(super) texture_debug_stripes_speed: f32,
    pub(super) directional_light_direction: [f32; 4],
    pub(super) ibl_blend_factor: f32,
    pub(super) oit_z_scale: f32,
    pub(super) _pad_pre_flat: [f32; 2],
    pub(super) flat_color: [f32; 4],
    pub(super) prev_view_projection: [[f32; 4]; 4],
    pub(super) cur_view_projection: [[f32; 4]; 4],
    pub(super) _padding3: [f32; 44],
}

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
    pub(super) flip_winding: u32,
    pub(super) entity_id: u32,
    pub(super) is_transparent: u32,
    pub(super) _padding: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct MorphDisplacement {
    pub(super) position: [f32; 3],
    pub(super) _pad0: f32,
    pub(super) normal: [f32; 3],
    pub(super) _pad1: f32,
    pub(super) tangent: [f32; 3],
    pub(super) _pad2: f32,
}

pub use super::super::material_gpu::MaterialData;

#[derive(Debug, Clone)]
pub(super) struct SkinnedMeshData {
    pub(super) vertex_offset: u32,
    pub(super) vertex_count: u32,
    pub(super) index_offset: u32,
    pub(super) index_count: u32,
    pub(super) _skin_index: u32,
    pub(super) morph_displacement_offset: u32,
    pub(super) morph_target_count: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct SkinnedCullObject {
    pub(super) bounds: [f32; 4],
    pub(super) command_index: u32,
    pub(super) _pad: [u32; 3],
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
    pub(super) _pad: [u32; 3],
}
