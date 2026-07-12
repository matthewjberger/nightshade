mod constructor;
mod materials;
mod mesh_registry;
mod uniforms;

use crate::skinning::SkinningCache;
use std::collections::{HashMap, HashSet};

use super::types::*;
use super::world_state::{SkinnedWorldGpuBuffers, SkinnedWorldRenderState};

pub(super) const SKINNED_OIT_CACHE_CAPACITY: usize = 4;

pub(super) struct SkinnedOitBundle {
    pub accum_texture: wgpu::Texture,
    pub accum_view: wgpu::TextureView,
    pub reveal_texture: wgpu::Texture,
    pub reveal_view: wgpu::TextureView,
}

pub struct SkinnedMeshPass {
    pub(super) skinning_compute_pipeline: wgpu::ComputePipeline,
    pub(super) skinning_compute_bind_group: wgpu::BindGroup,
    pub(super) _skinning_compute_bind_group_layout: wgpu::BindGroupLayout,

    pub(super) bone_transforms_buffer: wgpu::Buffer,
    pub(super) inverse_bind_matrices_buffer: wgpu::Buffer,
    pub(super) joint_matrices_buffer: wgpu::Buffer,
    pub(super) prev_joint_matrices_buffer: wgpu::Buffer,
    pub(super) skin_data_buffer: wgpu::Buffer,
    /// Last frame's collected bone world matrices. When a frame's
    /// freshly collected transforms are bit-identical, the bone upload
    /// and the skinning compute dispatch are both skipped so idle
    /// skeletons cost nothing on the CPU or GPU.
    pub(super) cached_palette_generation: u64,
    /// `static_generation` of the skinning snapshot the pass last uploaded, so a
    /// bump signals the skinned set changed and the static GPU buffers and cache
    /// mirror must be refreshed from `renderer_state.render_skinning`.
    pub(super) cached_skinning_generation: u64,
    /// GPU keyframe sampling and bone-hierarchy evaluation. Writes bone
    /// world matrices straight into `bone_transforms_buffer` for animated
    /// skeletons, replacing the CPU sampling and hierarchy walk.
    pub(super) animation: super::animation::SkinnedAnimationGpu,
    /// Bumped whenever `bone_transforms_buffer` is reallocated so the
    /// animation bind group knows to rebind the new buffer.
    pub(super) bone_transforms_generation: u64,
    pub(super) bone_transforms_buffer_size: usize,
    pub(super) inverse_bind_matrices_buffer_size: usize,
    pub(super) joint_matrices_buffer_size: usize,
    pub(super) skin_data_buffer_size: usize,

    pub(super) render_pipeline: wgpu::RenderPipeline,
    pub(super) oit_render_pipeline: wgpu::RenderPipeline,
    pub(super) blend_opaque_depth_prepass_pipeline: wgpu::RenderPipeline,
    pub(super) oit_composite_pipeline: wgpu::RenderPipeline,
    pub(super) oit_composite_bind_group_layout: wgpu::BindGroupLayout,

    pub(super) cull_pipeline: wgpu::ComputePipeline,
    pub(super) cull_bind_group_layout: wgpu::BindGroupLayout,

    pub(super) batch_clear_pipeline: wgpu::ComputePipeline,
    pub(super) batch_count_pipeline: wgpu::ComputePipeline,
    pub(super) batch_build_pipeline: wgpu::ComputePipeline,
    pub(super) batch_assign_pipeline: wgpu::ComputePipeline,
    pub(super) batch_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) batch_params_buffer: wgpu::Buffer,
    pub(super) batch_mesh_geo_buffer: wgpu::Buffer,
    pub(super) batch_mesh_geo_buffer_size: usize,
    pub(super) gpu_batching_enabled: bool,

    pub(super) uniform_buffer: wgpu::Buffer,
    pub(super) uniform_bind_group: wgpu::BindGroup,

    pub(super) instance_bind_group_layout: wgpu::BindGroupLayout,

    pub(super) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) shadow_bind_group: wgpu::BindGroup,
    pub(super) shadow_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) shadow_sampler: wgpu::Sampler,
    pub(super) shadow_pcf_sampler: wgpu::Sampler,
    pub(super) ltc: crate::wgpu::passes::geometry::ltc::LtcTextures,
    pub(super) spotlight_shadow_buffer: wgpu::Buffer,
    pub(super) point_shadow_buffer: wgpu::Buffer,
    pub(super) point_shadow_sampler: wgpu::Sampler,
    pub(super) point_shadow_cubemap_view: wgpu::TextureView,

    pub(super) dummy_white_view: wgpu::TextureView,
    pub(super) dummy_cube_view: wgpu::TextureView,
    pub(super) dummy_sampler: wgpu::Sampler,

    pub(super) brdf_lut_view: Option<wgpu::TextureView>,
    pub(super) irradiance_map_view: Option<wgpu::TextureView>,
    pub(super) prefiltered_env_view: Option<wgpu::TextureView>,
    pub(super) irradiance_b_view: Option<wgpu::TextureView>,
    pub(super) prefiltered_b_view: Option<wgpu::TextureView>,
    pub(super) ibl_sampler: wgpu::Sampler,

    pub(super) skinned_vertex_buffer: wgpu::Buffer,
    pub(super) skinned_index_buffer: wgpu::Buffer,
    pub(super) skinned_vertex_buffer_size: u64,
    pub(super) skinned_index_buffer_size: u64,

    pub(super) skinned_meshes: HashMap<String, u32>,
    pub(super) skinned_mesh_data: Vec<SkinnedMeshData>,
    pub(super) current_skinned_vertex_offset: u32,
    pub(super) current_skinned_index_offset: u32,

    pub(super) morph_displacement_buffer: wgpu::Buffer,
    pub(super) morph_displacement_buffer_size: usize,
    pub(super) current_morph_displacement_offset: u32,

    pub(super) total_joints_to_dispatch: u32,

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

    pub(super) skinning_cache: SkinningCache,

    pub(super) oit_accum_texture: wgpu::Texture,
    pub(super) oit_accum_view: wgpu::TextureView,
    pub(super) oit_reveal_texture: wgpu::Texture,
    pub(super) oit_reveal_view: wgpu::TextureView,
    pub(super) oit_sampler: wgpu::Sampler,
    pub(super) oit_texture_size: (u32, u32),
    pub(super) oit_resource_cache: Vec<((u32, u32), SkinnedOitBundle)>,

    pub(super) cluster_bounds_buffer: wgpu::Buffer,
    pub(super) view_matrix_buffer: wgpu::Buffer,
    pub(super) cluster_bounds_pipeline: wgpu::ComputePipeline,
    pub(super) cluster_assign_pipeline: wgpu::ComputePipeline,
    pub(super) cluster_bounds_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) cluster_assign_bind_group_layout: wgpu::BindGroupLayout,

    pub(super) current_world_id: u64,
    pub(super) world_states: Vec<Option<SkinnedWorldRenderState>>,
    pub(super) frame_counter: u64,
    pub dirty_skinned_mesh_names: HashSet<String>,
    pub(super) last_mesh_cache_clear_generation: u64,
}

fn skinned_texture_bind_group_layout_entries(
    bindless_max: Option<u32>,
) -> Vec<wgpu::BindGroupLayoutEntry> {
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

impl SkinnedMeshPass {
    pub fn set_current_world(&mut self, world_id: u64) {
        self.current_world_id = world_id;
        let index = world_id as usize;
        if self.world_states.len() <= index {
            self.world_states.resize_with(index + 1, || None);
        }
        if self.world_states[index].is_none() {
            self.world_states[index] = Some(SkinnedWorldRenderState::default());
        }
    }

    pub(super) fn state(&self) -> &SkinnedWorldRenderState {
        self.world_states[self.current_world_id as usize]
            .as_ref()
            .unwrap()
    }

    pub(super) fn state_mut(&mut self) -> &mut SkinnedWorldRenderState {
        self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap()
    }

    pub(super) fn gpu(&self) -> &SkinnedWorldGpuBuffers {
        self.state().gpu_buffers.as_ref().unwrap()
    }

    pub(super) fn gpu_mut(&mut self) -> &mut SkinnedWorldGpuBuffers {
        self.state_mut().gpu_buffers.as_mut().unwrap()
    }

    pub(super) fn ensure_world_gpu_buffers(&mut self, device: &wgpu::Device) {
        let world_state = self.world_states[self.current_world_id as usize]
            .as_mut()
            .unwrap();
        if world_state.gpu_buffers.is_none() {
            world_state.gpu_buffers = Some(SkinnedWorldGpuBuffers::new(
                device,
                &self.instance_bind_group_layout,
                &self.cull_bind_group_layout,
                &self.cluster_bounds_bind_group_layout,
                &self.cluster_bounds_buffer,
                (
                    &self.joint_matrices_buffer,
                    &self.prev_joint_matrices_buffer,
                ),
                &self.morph_displacement_buffer,
            ));
        }
    }
}
