use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
mod constructor;
mod materials;
mod mesh_registry;
mod uniforms;

use nightshade_render::data::skinning::SkinningCache;
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

pub(super) struct SkinnedSkinning {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,
    pub bone_transforms: wgpu::Buffer,
    pub publish_pending: bool,
    pub inverse_binds: wgpu::Buffer,
    pub joints: wgpu::Buffer,
    pub previous_joints: wgpu::Buffer,
    pub skin_data: wgpu::Buffer,
    pub palette_generation: u64,
    pub skinning_generation: u64,
    pub bone_generation: u64,
    pub bone_transforms_size: usize,
    pub inverse_binds_size: usize,
    pub joints_size: usize,
    pub skin_data_size: usize,
    pub joints_to_dispatch: u32,
    pub cache: SkinningCache,
}

pub(super) struct SkinnedOit {
    pub render_pipeline: wgpu::RenderPipeline,
    pub composite_pipeline: wgpu::RenderPipeline,
    pub composite_layout: wgpu::BindGroupLayout,
    pub accum_texture: wgpu::Texture,
    pub accum_view: wgpu::TextureView,
    pub reveal_texture: wgpu::Texture,
    pub reveal_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub texture_size: (u32, u32),
    pub resource_cache: Vec<((u32, u32), SkinnedOitBundle)>,
}

pub(super) struct SkinnedCompute {
    pub cull_pipeline: wgpu::ComputePipeline,
    pub cull_layout: wgpu::BindGroupLayout,
    pub bounds_pipeline: wgpu::ComputePipeline,
    pub bounds_layout: wgpu::BindGroupLayout,
    pub bounds_group: Option<wgpu::BindGroup>,
    pub bounds_dispatch: u32,
    pub batch_clear: wgpu::ComputePipeline,
    pub batch_count: wgpu::ComputePipeline,
    pub batch_build: wgpu::ComputePipeline,
    pub batch_assign: wgpu::ComputePipeline,
    pub batch_layout: wgpu::BindGroupLayout,
    pub batch_params: wgpu::Buffer,
    pub batch_geo: wgpu::Buffer,
    pub batch_geo_size: usize,
    pub batching_enabled: bool,
}

pub(super) struct SkinnedBindings {
    pub uniforms: wgpu::Buffer,
    pub uniform_group: wgpu::BindGroup,
    pub instance_layout: wgpu::BindGroupLayout,
    pub texture_layout: wgpu::BindGroupLayout,
    pub shadow_group: wgpu::BindGroup,
    pub shadow_layout: wgpu::BindGroupLayout,
    pub shadow_sampler: wgpu::Sampler,
    pub shadow_pcf_sampler: wgpu::Sampler,
    pub spotlight_shadows: wgpu::Buffer,
    pub point_shadows: wgpu::Buffer,
    pub point_shadow_sampler: wgpu::Sampler,
    pub point_shadow_cubemap: wgpu::TextureView,
    pub ibl_sampler: wgpu::Sampler,
}

pub(super) struct SkinnedImages {
    pub white: wgpu::TextureView,
    pub cube: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub brdf_lut: wgpu::TextureView,
    pub irradiance: Option<wgpu::TextureView>,
    pub prefiltered: Option<wgpu::TextureView>,
    pub irradiance_b: Option<wgpu::TextureView>,
    pub prefiltered_b: Option<wgpu::TextureView>,
}

pub(super) struct SkinnedBuffers {
    pub vertex: wgpu::Buffer,
    pub index: wgpu::Buffer,
    pub vertex_size: u64,
    pub index_size: u64,
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub ids: HashMap<String, u32>,
    pub data: Vec<SkinnedMeshData>,
}

pub(super) struct SkinnedMorph {
    pub buffer: wgpu::Buffer,
    pub buffer_size: usize,
    pub offset: u32,
}

pub(super) struct SkinnedMaterials {
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

pub(super) struct SkinnedClusters {
    pub bounds_buffer: wgpu::Buffer,
    pub view_matrix_buffer: wgpu::Buffer,
    pub bounds_pipeline: wgpu::ComputePipeline,
    pub assign_pipeline: wgpu::ComputePipeline,
    pub bounds_layout: wgpu::BindGroupLayout,
    pub assign_layout: wgpu::BindGroupLayout,
}

pub struct SkinnedMeshPass {
    pub(super) clusters: SkinnedClusters,
    pub(super) materials: SkinnedMaterials,
    pub(super) morph: SkinnedMorph,
    pub(super) buffers: SkinnedBuffers,
    pub(super) images: SkinnedImages,
    pub(super) bindings: SkinnedBindings,
    pub(super) compute: SkinnedCompute,
    pub(super) oit: SkinnedOit,
    pub(super) skinning: SkinnedSkinning,

    pub(super) animation: super::animation::SkinnedAnimationGpu,
    pub(super) ik: super::ik::SkinnedIkGpu,
    pub(super) multi_ik: super::multi_ik::SkinnedMultiIkGpu,
    pub(super) aim: super::aim::SkinnedAimGpu,
    pub(super) spring: super::spring::SkinnedSpringGpu,

    pub(super) render_pipeline: wgpu::RenderPipeline,
    pub(super) blend_opaque_depth_prepass_pipeline: wgpu::RenderPipeline,

    pub(super) ltc: nightshade_wgpu_shading::backend::ltc::LtcTextures,
    pub(super) air: nightshade_wgpu_shading::backend::fog::AirReads,

    pub(super) state: Option<SkinnedWorldRenderState>,
    pub(super) frame_counter: u64,
    pub dirty_skinned_mesh_names: HashSet<String>,
    pub(super) last_mesh_cache_clear_generation: u64,
}

fn skinned_texture_bind_group_layout_entries(
    bindless_max: Option<u32>,
) -> Vec<wgpu::BindGroupLayoutEntry> {
    use nightshade_wgpu_shading::backend::material_gpu::material_sampler_entries;
    let visibility = wgpu::ShaderStages::FRAGMENT;
    if let Some(max_textures) = bindless_max {
        let mut entries = vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: std::num::NonZeroU32::new(max_textures),
        }];
        entries.extend(material_sampler_entries(1, visibility));
        return entries;
    }
    let mut entries = vec![
        build::texture_at(
            0,
            wgpu::TextureSampleType::Float { filterable: true },
            wgpu::TextureViewDimension::D2Array,
            visibility,
        ),
        build::texture_at(
            1,
            wgpu::TextureSampleType::Float { filterable: true },
            wgpu::TextureViewDimension::D2Array,
            visibility,
        ),
    ];
    entries.extend(material_sampler_entries(2, visibility));
    entries
}

impl SkinnedMeshPass {
    pub fn ensure_state(&mut self) {
        if self.state.is_none() {
            self.state = Some(SkinnedWorldRenderState::default());
        }
    }

    pub(super) fn state(&self) -> &SkinnedWorldRenderState {
        self.state.as_ref().unwrap()
    }

    pub(super) fn state_mut(&mut self) -> &mut SkinnedWorldRenderState {
        self.state.as_mut().unwrap()
    }

    pub(super) fn gpu(&self) -> &SkinnedWorldGpuBuffers {
        self.state().gpu_buffers.as_ref().unwrap()
    }

    pub(super) fn gpu_mut(&mut self) -> &mut SkinnedWorldGpuBuffers {
        self.state_mut().gpu_buffers.as_mut().unwrap()
    }

    pub(super) fn ensure_gpu_buffers(&mut self, device: &wgpu::Device) {
        let world_state = self.state.as_mut().unwrap();
        if world_state.gpu_buffers.is_none() {
            world_state.gpu_buffers = Some(SkinnedWorldGpuBuffers::new(
                device,
                &self.bindings.instance_layout,
                &self.compute.cull_layout,
                &self.clusters.bounds_layout,
                &self.clusters.bounds_buffer,
                (&self.skinning.joints, &self.skinning.previous_joints),
                &self.morph.buffer,
            ));
        }
    }
}
