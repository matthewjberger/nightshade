mod batching;
mod bind_groups;
mod compute;
mod constructor;
mod incremental;
mod materials;
mod mesh_registry;
mod resize;
mod uniforms;

use std::collections::{HashMap, HashSet};

use crate::mesh_state::MeshRenderStateInner;

use super::types::{MeshBoundsData, MeshData, MeshLodInfo};
use super::world_state::{WorldGpuBuffers, WorldRenderState};

pub struct MeshPass {
    pub(super) opaque_pipeline: wgpu::RenderPipeline,
    pub(super) opaque_double_sided_pipeline: wgpu::RenderPipeline,
    pub(super) overlay_opaque_pipeline: wgpu::RenderPipeline,
    pub(super) overlay_opaque_double_sided_pipeline: wgpu::RenderPipeline,
    pub(super) oit_pipeline: wgpu::RenderPipeline,
    pub(super) oit_composite_pipeline: wgpu::RenderPipeline,

    pub(super) oit_accum_texture: wgpu::Texture,
    pub(super) oit_accum_view: wgpu::TextureView,
    pub(super) oit_reveal_texture: wgpu::Texture,
    pub(super) oit_reveal_view: wgpu::TextureView,
    pub(super) oit_composite_bind_group: wgpu::BindGroup,
    pub(super) oit_composite_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) oit_sampler: wgpu::Sampler,

    pub(super) overlay_depth_texture: wgpu::Texture,
    pub(super) overlay_depth_view: wgpu::TextureView,
    pub(super) oit_texture_size: (u32, u32),
    pub(super) oit_resource_cache: Vec<((u32, u32), OitResourceBundle)>,

    pub(super) transmission_color_texture: wgpu::Texture,
    pub(super) transmission_color_view: wgpu::TextureView,
    pub(super) transmission_sampler: wgpu::Sampler,
    pub(super) transmission_size: (u32, u32),
    pub(super) transmission_mip_views: Vec<wgpu::TextureView>,
    pub(super) transmission_downsample_pipeline: wgpu::RenderPipeline,
    pub(super) transmission_downsample_layout: wgpu::BindGroupLayout,
    pub(super) transmission_downsample_sampler: wgpu::Sampler,

    pub(super) uniform_buffer: wgpu::Buffer,
    pub(super) uniform_bind_group: wgpu::BindGroup,
    pub(super) overlay_uniform_buffer: wgpu::Buffer,
    pub(super) overlay_uniform_bind_group: wgpu::BindGroup,

    pub(super) instance_bind_group_layout: wgpu::BindGroupLayout,

    pub(super) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) scene_bind_group: wgpu::BindGroup,
    pub(super) scene_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) shadow_sampler: wgpu::Sampler,
    pub(super) shadow_pcf_sampler: wgpu::Sampler,
    pub(super) ltc: crate::wgpu::passes::geometry::ltc::LtcTextures,
    pub(super) spotlight_shadow_buffer: wgpu::Buffer,
    pub(super) point_shadow_buffer: wgpu::Buffer,
    pub(super) point_shadow_sampler: wgpu::Sampler,
    pub(super) ibl_sampler: wgpu::Sampler,

    pub(super) vertex_buffer: wgpu::Buffer,
    pub(super) vertex_buffer_generation: u64,
    pub(super) index_buffer: wgpu::Buffer,
    pub(super) vertex_buffer_size: u64,
    pub(super) index_buffer_size: u64,

    pub(super) meshes: HashMap<String, u32>,
    pub(super) mesh_data: Vec<MeshData>,
    pub(super) mesh_names: Vec<String>,

    pub(super) registered_textures: HashMap<String, (wgpu::TextureView, wgpu::Sampler)>,
    pub(super) material_array_srgb_view: Option<wgpu::TextureView>,
    pub(super) material_array_linear_view: Option<wgpu::TextureView>,
    pub(super) material_array_sampler: Option<wgpu::Sampler>,
    pub(super) material_bind_group: Option<wgpu::BindGroup>,
    pub(super) material_bindless_max: Option<u32>,
    pub(super) material_layer_map: HashMap<
        crate::asset_id::TextureId,
        crate::wgpu::material_texture_arrays::MaterialTextureLayer,
    >,

    pub(super) current_vertex_offset: u32,
    pub(super) current_index_offset: u32,

    pub(super) last_vertex_utilization: f32,
    pub(super) last_index_utilization: f32,

    pub(super) culling_pipeline: wgpu::ComputePipeline,
    pub(super) culling_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) culling_uniform_buffer: wgpu::Buffer,
    pub(super) culling_uniform_buffer_frustum: wgpu::Buffer,

    pub(super) compaction_pipeline: wgpu::ComputePipeline,
    pub(super) compaction_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) compaction_params_buffer: wgpu::Buffer,
    pub(super) supports_multi_draw_count: bool,

    pub(super) hiz: crate::wgpu::passes::geometry::HizPass,

    pub(super) indirect_build_pipeline: wgpu::ComputePipeline,
    pub(super) indirect_build_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) indirect_build_params_buffer: wgpu::Buffer,
    pub(super) mesh_geo_buffer: wgpu::Buffer,
    pub(super) mesh_geo_buffer_size: usize,

    pub(super) batch_assign_pipeline: wgpu::ComputePipeline,
    pub(super) batch_assign_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) batch_assign_params_buffer: wgpu::Buffer,

    pub(super) gpu_batch_classify_pipeline: wgpu::ComputePipeline,
    pub(super) gpu_batch_clear_pipeline: wgpu::ComputePipeline,
    pub(super) gpu_batch_count_pipeline: wgpu::ComputePipeline,
    pub(super) gpu_batch_build_pipeline: wgpu::ComputePipeline,
    pub(super) gpu_batch_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) gpu_batch_params_buffer: wgpu::Buffer,
    pub(super) mesh_lod_geo_buffer: wgpu::Buffer,
    pub(super) mesh_lod_geo_buffer_size: usize,
    pub(super) gpu_batching_enabled: bool,
    pub(super) gpu_batch_dirty: bool,
    pub(super) gpu_batch_cap: usize,

    pub(super) mesh_bounds_buffer: wgpu::Buffer,
    pub(super) mesh_bounds_buffer_size: usize,
    pub(super) mesh_bounds_data: Vec<MeshBoundsData>,
    pub(super) mesh_lod_buffer: wgpu::Buffer,
    pub(super) mesh_lod_buffer_size: usize,
    pub(super) mesh_lod_data: Vec<MeshLodInfo>,
    pub(super) mesh_lod_mesh_ids: Vec<Vec<u32>>,

    pub(super) depth_prepass_pipeline: wgpu::RenderPipeline,
    pub(super) blend_opaque_depth_prepass_pipeline: wgpu::RenderPipeline,

    pub(super) morph_displacement_buffer: wgpu::Buffer,
    pub(super) morph_displacement_buffer_size: u64,
    pub(super) current_morph_displacement_offset: u32,

    pub(super) brdf_lut_view: wgpu::TextureView,
    pub(super) irradiance_view: wgpu::TextureView,
    pub(super) prefiltered_view: wgpu::TextureView,
    pub(super) irradiance_b_view: wgpu::TextureView,
    pub(super) prefiltered_b_view: wgpu::TextureView,
    pub(super) point_shadow_cubemap_view: wgpu::TextureView,

    pub(super) last_prepared_world_id: Option<u64>,
    pub(super) current_world_id: u64,
    pub(super) world_states: Vec<Option<WorldRenderState>>,
    pub(super) frame_counter: u64,
    pub(super) resize_full_rebuild_pending: bool,

    pub(super) cluster_bounds_buffer: wgpu::Buffer,
    pub(super) cluster_bounds_pipeline: wgpu::ComputePipeline,
    pub(super) cluster_assign_pipeline: wgpu::ComputePipeline,
    pub(super) _cluster_bounds_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) cluster_assign_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) view_matrix_buffer: wgpu::Buffer,

    pub(super) instanced_compute_pipeline: wgpu::ComputePipeline,
    pub(super) instanced_compute_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) instanced_compute_uniforms_buffer: wgpu::Buffer,
    pub(super) instanced_compute_staging_buffer: wgpu::Buffer,
    pub(super) scene_bind_group_dirty: bool,
    pub(super) scratch_mesh_updates: Vec<(u32, super::types::ModelMatrix)>,
    pub frame_dirty: Option<MeshRenderStateInner>,
    /// Whether the pass's prepare ran since `frame_dirty` was last handed
    /// over. When false, the next handoff merges instead of replacing so a
    /// skipped or compose-only frame's dirty marks are not lost.
    pub frame_dirty_consumed: bool,
    pub dirty_mesh_ids: HashSet<u32>,
    pub(super) last_mesh_cache_clear_generation: u64,
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

fn texture_bind_group_layout_entries(bindless_max: Option<u32>) -> Vec<wgpu::BindGroupLayoutEntry> {
    if let Some(max_textures) = bindless_max {
        return vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: std::num::NonZeroU32::new(max_textures),
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];
    }
    vec![
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
    ]
}

impl MeshPass {
    #[inline]
    pub(super) fn state(&self) -> &WorldRenderState {
        self.world_states[self.current_world_id as usize]
            .as_ref()
            .unwrap()
    }

    #[inline]
    pub(super) fn state_mut(&mut self) -> &mut WorldRenderState {
        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
    }

    #[inline]
    pub(super) fn gpu(&self) -> &WorldGpuBuffers {
        self.state().gpu_buffers.as_ref().unwrap()
    }

    #[inline]
    pub(super) fn gpu_mut(&mut self) -> &mut WorldGpuBuffers {
        self.state_mut().gpu_buffers.as_mut().unwrap()
    }

    #[inline]
    pub(super) fn world_state_get(&self, world_id: u64) -> Option<&WorldRenderState> {
        self.world_states
            .get(world_id as usize)
            .and_then(|slot| slot.as_ref())
    }

    #[inline]
    pub(super) fn world_state_get_mut(&mut self, world_id: u64) -> Option<&mut WorldRenderState> {
        self.world_states
            .get_mut(world_id as usize)
            .and_then(|slot| slot.as_mut())
    }

    pub(super) fn world_state_ensure(&mut self, world_id: u64) {
        let index = world_id as usize;
        if self.world_states.len() <= index {
            self.world_states.resize_with(index + 1, || None);
        }
        if self.world_states[index].is_none() {
            self.world_states[index] = Some(WorldRenderState::new());
        }
    }

    pub(super) fn world_state_remove(&mut self, world_id: u64) {
        if let Some(slot) = self.world_states.get_mut(world_id as usize) {
            *slot = None;
        }
    }

    pub(super) fn world_state_ids(&self) -> Vec<u64> {
        self.world_states
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_ref().map(|_| index as u64))
            .collect()
    }
}
