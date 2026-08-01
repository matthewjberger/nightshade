use nightshade_wgpu::wgpu;
mod batching;
mod bind_groups;
mod compute;
mod constructor;
mod incremental;
pub(super) use incremental::debug_view_exempt;
mod materials;
mod mesh_registry;
mod resize;
mod uniforms;

use std::collections::{HashMap, HashSet};

use nightshade_render::resources::mesh_state::MeshRenderState;

use super::types::{MeshBoundsData, MeshData};
use super::world_state::{WorldGpuBuffers, WorldRenderState};

pub(super) struct MeshOit {
    pub pipeline: wgpu::RenderPipeline,
    pub composite_pipeline: wgpu::RenderPipeline,
    pub accum_texture: wgpu::Texture,
    pub accum_view: wgpu::TextureView,
    pub reveal_texture: wgpu::Texture,
    pub reveal_view: wgpu::TextureView,
    pub composite_bind_group: wgpu::BindGroup,
    pub composite_bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub texture_size: (u32, u32),
    pub resource_cache: Vec<((u32, u32), OitResourceBundle)>,
}

pub struct MeshTransmission {
    pub publish_pending: bool,
    pub color_texture: wgpu::Texture,
    pub color_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: (u32, u32),
    pub mip_views: Vec<wgpu::TextureView>,
    pub downsample_pipeline: wgpu::RenderPipeline,
    pub downsample_layout: wgpu::BindGroupLayout,
    pub downsample_sampler: wgpu::Sampler,
}

pub(super) struct MeshBuffers {
    pub vertex: wgpu::Buffer,
    pub index: wgpu::Buffer,
    pub vertex_size: u64,
    pub index_size: u64,
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub vertex_utilization: f32,
    pub index_utilization: f32,
    pub free_vertex: Vec<(u32, u32)>,
    pub free_index: Vec<(u32, u32)>,
}

pub(super) struct MeshRegistry {
    pub ids: HashMap<u64, u32>,
    pub data: Vec<MeshData>,
    pub names: Vec<String>,
    pub free_ids: Vec<u32>,
    pub rescan: bool,
    pub new_names: Vec<String>,
    pub geo_buffer: wgpu::Buffer,
    pub geo_buffer_size: usize,
    pub geo_dirty: bool,
    pub bounds_buffer: wgpu::Buffer,
    pub bounds_buffer_size: usize,
    pub bounds_data: Vec<MeshBoundsData>,
}

pub(super) struct MeshMaterials {
    pub registered_textures: HashMap<String, (wgpu::TextureView, wgpu::Sampler)>,
    pub array_srgb_view: Option<wgpu::TextureView>,
    pub array_linear_view: Option<wgpu::TextureView>,
    pub array_samplers: Option<nightshade_wgpu_shading::backend::material_gpu::MaterialSamplers>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub bindless_max: Option<u32>,
    pub layer_map: HashMap<
        nightshade_render::data::asset_id::TextureId,
        nightshade_wgpu::backend::MaterialTextureLayer,
    >,
}

pub(super) struct MeshCompute {
    pub cull_pipeline: wgpu::ComputePipeline,
    pub cull_layout: wgpu::BindGroupLayout,
    pub cull_uniforms: wgpu::Buffer,
    pub cull_uniforms_frustum: wgpu::Buffer,
    pub compaction_pipeline: wgpu::ComputePipeline,
    pub compaction_layout: wgpu::BindGroupLayout,
    pub compaction_params: wgpu::Buffer,
    pub supports_multi_draw_count: bool,
    pub indirect_pipeline: wgpu::ComputePipeline,
    pub indirect_layout: wgpu::BindGroupLayout,
    pub indirect_params: wgpu::Buffer,
    pub assign_pipeline: wgpu::ComputePipeline,
    pub assign_layout: wgpu::BindGroupLayout,
    pub assign_params: wgpu::Buffer,
    pub batch_classify: wgpu::ComputePipeline,
    pub batch_clear: wgpu::ComputePipeline,
    pub batch_count: wgpu::ComputePipeline,
    pub batch_build: wgpu::ComputePipeline,
    pub batch_layout: wgpu::BindGroupLayout,
    pub batch_params: wgpu::Buffer,
    pub batching_enabled: bool,
    pub batch_dirty: bool,
    pub batch_cap: usize,
    pub batch_keys_per_lane: usize,
}

pub(super) struct MeshClusters {
    pub bounds_buffer: wgpu::Buffer,
    pub bounds_pipeline: wgpu::ComputePipeline,
    pub assign_pipeline: wgpu::ComputePipeline,
    pub bounds_layout: wgpu::BindGroupLayout,
    pub assign_layout: wgpu::BindGroupLayout,
    pub view_matrix_buffer: wgpu::Buffer,
}

pub(super) struct MeshImageLighting {
    pub brdf_lut: wgpu::TextureView,
    pub irradiance: wgpu::TextureView,
    pub prefiltered: wgpu::TextureView,
    pub irradiance_b: wgpu::TextureView,
    pub prefiltered_b: wgpu::TextureView,
    pub point_shadow_cubemap: wgpu::TextureView,
}

pub(super) struct MeshPipelines {
    pub opaque: wgpu::RenderPipeline,
    pub opaque_double_sided: wgpu::RenderPipeline,
    pub overlay: wgpu::RenderPipeline,
    pub overlay_double_sided: wgpu::RenderPipeline,
    pub depth_prepass: wgpu::RenderPipeline,
    pub blend_depth_prepass: wgpu::RenderPipeline,
    pub overlay_depth_texture: wgpu::Texture,
    pub overlay_depth_view: wgpu::TextureView,
}

pub(super) struct MeshBindings {
    pub uniforms: wgpu::Buffer,
    pub uniform_group: wgpu::BindGroup,
    pub overlay_uniforms: wgpu::Buffer,
    pub overlay_uniform_group: wgpu::BindGroup,
    pub instance_layout: wgpu::BindGroupLayout,
    pub texture_layout: wgpu::BindGroupLayout,
    pub scene_group: wgpu::BindGroup,
    pub scene_layout: wgpu::BindGroupLayout,
    pub scene_group_dirty: bool,
    pub shadow_sampler: wgpu::Sampler,
    pub shadow_pcf_sampler: wgpu::Sampler,
    pub spotlight_shadows: wgpu::Buffer,
    pub point_shadows: wgpu::Buffer,
    pub point_shadow_sampler: wgpu::Sampler,
    pub ibl_sampler: wgpu::Sampler,
}

pub(super) struct MeshInstanced {
    pub pipeline: wgpu::ComputePipeline,
    pub layout: wgpu::BindGroupLayout,
    pub uniforms: wgpu::Buffer,
    pub uniform_slots: usize,
}

pub(super) struct MeshMorph {
    pub buffer: wgpu::Buffer,
    pub buffer_size: u64,
    pub offset: u32,
}

pub struct MeshFrame {
    pub counter: u64,
    pub last_compaction: u64,
    pub resize_rebuild_pending: bool,
    pub(super) scratch_updates: Vec<(u32, super::types::ModelMatrix)>,
    pub dirty: Option<MeshRenderState>,
    pub dirty_consumed: bool,
    pub dirty_mesh_ids: HashSet<u32>,
    pub last_cache_clear: u64,
}

pub struct MeshPass {
    pub frame: MeshFrame,
    pub(super) morph: MeshMorph,
    pub(super) instanced: MeshInstanced,
    pub(super) bindings: MeshBindings,
    pub(super) pipelines: MeshPipelines,
    pub(super) image_lighting: MeshImageLighting,
    pub(super) clusters: MeshClusters,
    pub(super) compute: MeshCompute,
    pub(super) materials: MeshMaterials,
    pub(super) registry: MeshRegistry,
    pub(super) buffers: MeshBuffers,
    pub transmission: MeshTransmission,
    pub(super) oit: MeshOit,

    pub(super) ltc: nightshade_wgpu_shading::backend::ltc::LtcTextures,

    pub(super) hiz: nightshade_wgpu_shading::backend::hiz::HizPass,
    pub air: nightshade_wgpu_shading::backend::fog::Air,

    pub(super) state: Option<WorldRenderState>,
}

pub(super) const OIT_RESOURCE_CACHE_CAPACITY: usize = 4;

pub(super) struct OitResourceBundle {
    pub accum_texture: wgpu::Texture,
    pub accum_view: wgpu::TextureView,
    pub reveal_texture: wgpu::Texture,
    pub reveal_view: wgpu::TextureView,
    pub overlay_depth_texture: wgpu::Texture,
    pub overlay_depth_view: wgpu::TextureView,
    pub composite_bind_group: wgpu::BindGroup,
}

impl MeshPass {
    #[inline]
    pub(super) fn state(&self) -> &WorldRenderState {
        self.state.as_ref().unwrap()
    }

    #[inline]
    pub(super) fn state_mut(&mut self) -> &mut WorldRenderState {
        self.state.as_mut().unwrap()
    }

    #[inline]
    pub(super) fn gpu(&self) -> &WorldGpuBuffers {
        self.state().gpu_buffers.as_ref().unwrap()
    }

    #[inline]
    pub(super) fn gpu_mut(&mut self) -> &mut WorldGpuBuffers {
        self.state_mut().gpu_buffers.as_mut().unwrap()
    }
}
