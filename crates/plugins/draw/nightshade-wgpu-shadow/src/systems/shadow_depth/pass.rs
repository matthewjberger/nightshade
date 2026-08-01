use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

fn take_span(free: &mut Vec<(u32, u32)>, high: &mut u32, count: u32) -> u32 {
    if count == 0 {
        return *high;
    }
    if let Some(index) = free.iter().position(|(_, size)| *size >= count) {
        let (offset, size) = free.remove(index);
        if size > count {
            free.push((offset + count, size - count));
        }
        return offset;
    }
    let offset = *high;
    *high += count;
    offset
}

fn give_span(free: &mut Vec<(u32, u32)>, offset: u32, count: u32) {
    if count > 0 {
        free.push((offset, count));
    }
}
use super::types::ShadowVertex;
use nightshade_render::data::generational_registry::registry_entry_by_name;
use nightshade_render::data::geometry::{SkinnedVertex, Vertex};
use nightshade_render::data::skinning::GpuSkinData;
use nightshade_render::queries::mesh_cache::mesh_cache_iter;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

use nightshade_render::data::asset_id::TextureId;
use nightshade_render::data::config::RenderMaterials;
use nightshade_render::data::material::AlphaMode;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::grow_storage_buffer;
use nightshade_wgpu::backend::{BindlessConfig, MaterialTextureArrays, MaterialTextureLayer};
use nightshade_wgpu_shading::backend::material_gpu::{
    self, MaterialData, convert_material_to_gpu_data, material_texture_bind_group_layout_entries,
};

const BUFFER_CEILING: u64 = 1024 * 1024 * 1024;

const SHADOW_CUTOUT_TEST_LOSES_THE_DEVICE: bool = cfg!(target_os = "android");

pub(super) fn cutout_fragment_entry(entry_point: &'static str) -> Option<&'static str> {
    if SHADOW_CUTOUT_TEST_LOSES_THE_DEVICE {
        return None;
    }
    Some(entry_point)
}

pub(super) fn shadow_color_fragment_entry(with_cutout: &'static str) -> &'static str {
    if SHADOW_CUTOUT_TEST_LOSES_THE_DEVICE {
        "fs_distance"
    } else {
        with_cutout
    }
}

const COMPACTION_CHECK_INTERVAL_FRAMES: u64 = 300;
const COMPACTION_UTILIZATION_THRESHOLD: f32 = 0.5;
const COMPACTION_MIN_RECLAIM_BYTES: u64 = 8 * 1024 * 1024;

pub(super) const SKINNED_CULL_MAX_VIEWS: usize =
    super::types::NUM_SHADOW_CASCADES + super::types::MAX_SPOTLIGHT_SHADOWS;

use super::types::{
    CASCADE_SPLIT_DISTANCES, CascadeData, INITIAL_MAX_SKINS, INITIAL_MAX_TOTAL_BONES,
    MAX_JOINTS_PER_SKIN, MAX_POINT_LIGHT_SHADOWS, MAX_SPOTLIGHT_SHADOWS, NUM_SHADOW_CASCADES,
    POINT_SHADOW_FACE_SIZE, POINT_SHADOW_NUM_FACES, PointLightShadowData, PointLightShadowSlot,
    PointShadowUniforms, ShadowSlotCache, ShadowUniforms, SkinnedShadowObjectData,
    SpotlightShadowData, SpotlightShadowSlot,
};

pub(super) struct ShadowGeometry {
    pub vertex: wgpu::Buffer,
    pub index: wgpu::Buffer,
    pub vertex_size: u64,
    pub index_size: u64,
    pub ids: HashMap<String, (u32, u32, u32, u32)>,
    pub oversized: std::collections::HashSet<String>,
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub free_vertex: Vec<(u32, u32)>,
    pub free_index: Vec<(u32, u32)>,
    pub compaction_counter: u64,
    pub crowded_at: u64,
}

pub(super) struct ShadowOccluders {
    pub rows: Vec<super::types::ShadowOccluder>,
    pub transforms: Vec<[[f32; 4]; 4]>,
    pub cache: Vec<super::node::OccluderCacheEntry>,
    pub slot_of: HashMap<nightshade::ecs::Entity, u32>,
    pub pending_casters: Vec<nightshade::ecs::Entity>,
    pub instanced_casters: std::collections::HashSet<nightshade::ecs::Entity>,
    pub casters: Vec<nightshade::ecs::Entity>,
    pub frame_changes: super::types::ShadowFrameChanges,
    pub seen_geo_count: usize,
    pub initialized: bool,
    pub mesh_generation: u64,
}

pub(super) struct ShadowSkinning {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,
    pub joints: wgpu::Buffer,
    pub bone_transforms: wgpu::Buffer,
    pub inverse_binds: wgpu::Buffer,
    pub skin_data: wgpu::Buffer,
    pub bone_transforms_size: usize,
    pub inverse_binds_size: usize,
    pub joints_size: usize,
    pub skin_data_size: usize,
    pub bone_source_external: bool,
    pub bone_bound: bool,
    pub joints_to_dispatch: u32,
}

pub(super) struct ShadowSkinnedGeometry {
    pub vertex: wgpu::Buffer,
    pub index: wgpu::Buffer,
    pub vertex_size: u64,
    pub index_size: u64,
    pub ids: HashMap<String, (u32, u32, u32, u32)>,
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub compaction_counter: u64,
    pub casters: Vec<nightshade::ecs::Entity>,
}

pub(super) struct ShadowSkinnedDraw {
    pub pipeline: wgpu::RenderPipeline,
    pub objects: wgpu::Buffer,
    pub objects_size: usize,
    pub indirect: wgpu::Buffer,
    pub indirect_size: usize,
    pub group_count: usize,
    pub cull_view_count: usize,
    pub occluder_count: usize,
    pub cull_objects: wgpu::Buffer,
    pub cull_objects_size: usize,
    pub visible_indices: wgpu::Buffer,
    pub visible_indices_size: usize,
    pub cull_pipeline: wgpu::ComputePipeline,
    pub cull_layout: wgpu::BindGroupLayout,
    pub cull_uniforms: Vec<wgpu::Buffer>,
    pub cull_groups: Vec<wgpu::BindGroup>,
    pub bind_group: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,
}

pub(super) struct ShadowSpotlights {
    pub slots: Vec<SpotlightShadowSlot>,
    pub data: Vec<SpotlightShadowData>,
    pub atlas_slot_size: u32,
    pub slot_cache: [ShadowSlotCache; MAX_SPOTLIGHT_SHADOWS],
}

pub(super) struct ShadowPointLights {
    pub cubemap: wgpu::TextureView,
    pub publish_pending: bool,
    pub faces: Vec<wgpu::TextureView>,
    pub depth: wgpu::TextureView,
    pub uniforms: wgpu::Buffer,
    pub uniform_group: wgpu::BindGroup,
    pub slots: Vec<PointLightShadowSlot>,
    pub data: Vec<PointLightShadowData>,
    pub slot_cache: [ShadowSlotCache; MAX_POINT_LIGHT_SHADOWS],
}

pub(super) struct ShadowMaterials {
    pub texture_layout: wgpu::BindGroupLayout,
    pub bindless_max: Option<u32>,
    pub bind_group: Option<wgpu::BindGroup>,
    pub layer_map: HashMap<TextureId, MaterialTextureLayer>,
    pub layer_generation: u64,
    pub buffer: wgpu::Buffer,
    pub buffer_size: usize,
    pub indices: HashMap<u32, u32>,
    pub generation: u64,
    pub row_layer_generation: u64,
    pub class_generation: u64,
}

pub(super) struct ShadowCascades {
    pub data: CascadeData,
    pub uniforms: [wgpu::Buffer; NUM_SHADOW_CASCADES],
    pub uniform_groups: [wgpu::BindGroup; NUM_SHADOW_CASCADES],
}

pub(super) struct ShadowTransforms {
    pub uniforms: wgpu::Buffer,
    pub uniform_layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,
    pub buffer_size: usize,
}

pub struct ShadowDepthPass {
    pub(super) transforms: ShadowTransforms,
    pub(super) cascades: ShadowCascades,
    pub(super) materials: ShadowMaterials,
    pub(super) point_lights: ShadowPointLights,
    pub(super) spotlights: ShadowSpotlights,
    pub(super) skinned_draw: ShadowSkinnedDraw,
    pub(super) skinned_geometry: ShadowSkinnedGeometry,
    pub(super) skinning: ShadowSkinning,
    pub(super) occluders: ShadowOccluders,
    pub(super) geometry: ShadowGeometry,

    pub dirty_mesh_names: std::collections::HashSet<String>,

    pub shadow_scene_dirty: bool,

    pub(super) was_enabled_last_frame: bool,
    pub(super) shadow_view_dirty: Vec<bool>,
    pub(super) depth_clear_pipeline: wgpu::RenderPipeline,

    pub(super) culling: super::culling::ShadowCulling,
    pub(super) buffer_cap: u64,
}

fn buffer_bind_group(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::BindGroupLayout,
    buffers: &[&wgpu::Buffer],
) -> wgpu::BindGroup {
    let entries: Vec<wgpu::BindGroupEntry> = buffers
        .iter()
        .enumerate()
        .map(|(index, buffer)| wgpu::BindGroupEntry {
            binding: index as u32,
            resource: buffer.as_entire_binding(),
        })
        .collect();
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &entries,
    })
}

impl ShadowDepthPass {
    pub fn new(
        build: PassBuild<'_>,
        queue: &wgpu::Queue,
        bindless_config: BindlessConfig,
        point_shadow_format: wgpu::TextureFormat,
    ) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let material_bindless_max = bindless_config
            .enabled
            .then_some(bindless_config.max_textures);
        let material_shader_defs: &[(&str, naga_oil::compose::ShaderDefValue)] =
            if bindless_config.enabled {
                &[("BINDLESS", naga_oil::compose::ShaderDefValue::Bool(true))]
            } else {
                &[]
            };
        let texture_bind_group_layout = build::bind_layout(
            device,
            "Shadow Material Texture Bind Group Layout",
            &material_texture_bind_group_layout_entries(material_bindless_max),
        );
        let material_buffer_size = 16;
        let material_buffer = build::storage(
            device,
            "Shadow Material Buffer",
            (std::mem::size_of::<MaterialData>() * material_buffer_size) as u64,
        );
        let uniform_buffer = build::uniform(
            device,
            "Shadow Uniform Buffer",
            std::mem::size_of::<ShadowUniforms>() as u64,
        );

        let uniform_bind_group_layout = build::bind_layout(
            device,
            "Shadow Uniform Bind Group Layout",
            &[build::uniform_at(0, wgpu::ShaderStages::VERTEX_FRAGMENT)],
        );

        let initial_cascade_uniforms = ShadowUniforms {
            light_view_projection: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            time: 0.0,
            padding_time: [0.0; 3],
            padding: [[0.0; 4]; 4],
            padding_2: [[0.0; 4]; 4],
            padding_3: [[0.0; 4]; 4],
        };
        let cascade_uniform_buffers: [wgpu::Buffer; NUM_SHADOW_CASCADES] =
            std::array::from_fn(|_index| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Cascade Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[initial_cascade_uniforms]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                })
            });

        let cascade_uniform_bind_groups: [wgpu::BindGroup; NUM_SHADOW_CASCADES] =
            std::array::from_fn(|index| {
                build::binding(
                    device,
                    &format!("Cascade {} Uniform Bind Group", index),
                    &uniform_bind_group_layout,
                    vec![cascade_uniform_buffers[index].as_entire_binding()],
                )
            });

        let initial_transform_count = 1000;
        let transform_buffer = build::storage(
            device,
            "Shadow Transform Buffer",
            (std::mem::size_of::<[[f32; 4]; 4]>() * initial_transform_count) as u64,
        );

        let transform_bind_group_layout = build::bind_layout(
            device,
            "Shadow Transform Bind Group Layout",
            &[build::storage_at(0, false, wgpu::ShaderStages::VERTEX)],
        );

        let transform_bind_group = build::binding(
            device,
            "Shadow Transform Bind Group",
            &transform_bind_group_layout,
            vec![transform_buffer.as_entire_binding()],
        );

        let initial_vertex_count = 10000;
        let initial_index_count = 30000;

        let vertex_buffer = build::buffer(
            device,
            "Shadow Vertex Buffer",
            (std::mem::size_of::<ShadowVertex>() * initial_vertex_count) as u64,
            wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        let index_buffer = build::buffer(
            device,
            "Shadow Index Buffer",
            (std::mem::size_of::<u32>() * initial_index_count) as u64,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        );

        let bone_transforms_buffer = build::storage(
            device,
            "Shadow Bone Transforms Buffer",
            (std::mem::size_of::<[[f32; 4]; 4]>() * INITIAL_MAX_TOTAL_BONES) as u64,
        );

        let inverse_bind_matrices_buffer = build::storage(
            device,
            "Shadow Inverse Bind Matrices Buffer",
            (std::mem::size_of::<[[f32; 4]; 4]>() * MAX_JOINTS_PER_SKIN * INITIAL_MAX_SKINS) as u64,
        );

        let joint_matrices_buffer = build::storage(
            device,
            "Shadow Joint Matrices Buffer",
            (std::mem::size_of::<[[f32; 4]; 4]>() * MAX_JOINTS_PER_SKIN * INITIAL_MAX_SKINS) as u64,
        );

        let skin_data_buffer = build::storage(
            device,
            "Shadow Skin Data Buffer",
            (std::mem::size_of::<GpuSkinData>() * INITIAL_MAX_SKINS) as u64,
        );

        let skinning_compute_shader = composer.compile_wgsl(
            device,
            "skinning_compute.wgsl",
            nightshade_wgpu_shading::backend::SKINNING_COMPUTE,
        );

        let skinning_compute_bind_group_layout = build::bind_layout(
            device,
            "Shadow Skinning Compute Bind Group Layout",
            &[
                build::storage_at(0, false, wgpu::ShaderStages::COMPUTE),
                build::storage_at(1, false, wgpu::ShaderStages::COMPUTE),
                build::storage_at(2, false, wgpu::ShaderStages::COMPUTE),
                build::storage_at(3, true, wgpu::ShaderStages::COMPUTE),
            ],
        );

        let initial_joint_matrices_size = MAX_JOINTS_PER_SKIN * INITIAL_MAX_SKINS;

        let skinning_compute_bind_group = buffer_bind_group(
            device,
            "Shadow Skinning Compute Bind Group",
            &skinning_compute_bind_group_layout,
            &[
                &bone_transforms_buffer,
                &inverse_bind_matrices_buffer,
                &skin_data_buffer,
                &joint_matrices_buffer,
            ],
        );

        let skinning_compute_pipeline_layout = build::pipeline_layout(
            device,
            "Shadow Skinning Compute Pipeline Layout",
            &[&skinning_compute_bind_group_layout],
        );

        let skinning_compute_pipeline = build::compute(
            device,
            "Shadow Skinning Compute Pipeline",
            &skinning_compute_shader,
            "main",
            &skinning_compute_pipeline_layout,
        );

        let initial_skinned_object_count = 100;
        let skinned_object_buffer = build::storage(
            device,
            "Shadow Skinned Object Buffer",
            (std::mem::size_of::<SkinnedShadowObjectData>() * initial_skinned_object_count) as u64,
        );

        let initial_skinned_indirect_count = 32;
        let skinned_indirect_buffer = build::buffer(
            device,
            "Shadow Skinned Indirect Buffer",
            (std::mem::size_of::<super::types::ShadowDrawIndexedIndirect>()
                * initial_skinned_indirect_count) as u64,
            wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        );

        let initial_skinned_cull_objects = 100;
        let skinned_cull_objects_buffer = build::storage(
            device,
            "Shadow Skinned Cull Objects Buffer",
            (std::mem::size_of::<super::types::SkinnedShadowCullObject>()
                * initial_skinned_cull_objects) as u64,
        );
        let initial_skinned_visible = 100;
        let skinned_visible_indices_buffer = build::storage(
            device,
            "Shadow Skinned Visible Indices Buffer",
            (std::mem::size_of::<u32>() * initial_skinned_visible) as u64,
        );

        let skinned_cull_storage_entry =
            |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            };
        let skinned_cull_bind_group_layout = build::bind_layout(
            device,
            "Shadow Skinned Cull Bind Group Layout",
            &[
                skinned_cull_storage_entry(0, true),
                skinned_cull_storage_entry(1, false),
                skinned_cull_storage_entry(2, false),
                build::uniform_at(3, wgpu::ShaderStages::COMPUTE),
            ],
        );
        let skinned_cull_shader = composer.compile_wgsl(
            device,
            "skinned_shadow_cull.wgsl",
            include_str!("../../shaders/skinned_shadow_cull.wgsl"),
        );
        let skinned_cull_pipeline_layout = build::pipeline_layout(
            device,
            "Shadow Skinned Cull Pipeline Layout",
            &[&skinned_cull_bind_group_layout],
        );
        let skinned_cull_pipeline = build::compute(
            device,
            "Shadow Skinned Cull Pipeline",
            &skinned_cull_shader,
            "main",
            &skinned_cull_pipeline_layout,
        );
        let skinned_cull_uniform_buffers: Vec<wgpu::Buffer> = (0..SKINNED_CULL_MAX_VIEWS)
            .map(|view| {
                build::uniform(
                    device,
                    &format!("Shadow Skinned Cull Uniform Buffer {view}"),
                    std::mem::size_of::<super::types::ShadowCullUniforms>() as u64,
                )
            })
            .collect();
        let skinned_cull_bind_groups: Vec<wgpu::BindGroup> = skinned_cull_uniform_buffers
            .iter()
            .map(|uniform_buffer| {
                buffer_bind_group(
                    device,
                    "Shadow Skinned Cull Bind Group",
                    &skinned_cull_bind_group_layout,
                    &[
                        &skinned_cull_objects_buffer,
                        &skinned_indirect_buffer,
                        &skinned_visible_indices_buffer,
                        uniform_buffer,
                    ],
                )
            })
            .collect();

        let skinned_bind_group_layout = build::bind_layout(
            device,
            "Shadow Skinned Bind Group Layout",
            &[
                build::storage_at(0, false, wgpu::ShaderStages::VERTEX),
                build::storage_at(1, false, wgpu::ShaderStages::VERTEX),
                build::storage_at(2, false, wgpu::ShaderStages::VERTEX),
                build::storage_at(3, false, wgpu::ShaderStages::FRAGMENT),
            ],
        );

        let skinned_bind_group = buffer_bind_group(
            device,
            "Shadow Skinned Bind Group",
            &skinned_bind_group_layout,
            &[
                &skinned_object_buffer,
                &joint_matrices_buffer,
                &skinned_visible_indices_buffer,
                &material_buffer,
            ],
        );

        let skinned_shadow_shader = composer.compile_wgsl_with_defs(
            device,
            "skinned_shadow_depth.wgsl",
            include_str!("../../shaders/skinned_shadow_depth.wgsl"),
            material_shader_defs,
        );

        let skinned_pipeline_layout = build::pipeline_layout(
            device,
            "Skinned Shadow Pipeline Layout",
            &[
                &uniform_bind_group_layout,
                &skinned_bind_group_layout,
                &texture_bind_group_layout,
            ],
        );

        let skinned_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SkinnedVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 40,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 56,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 72,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Uint32x4,
                },
                wgpu::VertexAttribute {
                    offset: 88,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        };

        let mut skinned_pipe = build::Pipe::new(
            device,
            "Skinned Shadow Depth Pipeline",
            &skinned_shadow_shader,
        )
        .layout(&skinned_pipeline_layout)
        .buffers(std::slice::from_ref(&skinned_vertex_layout))
        .depth(build::depth_state(wgpu::TextureFormat::Depth32Float, true))
        .cull_front()
        .vertex("vs_main");
        if let Some(entry) = cutout_fragment_entry("fs_main") {
            skinned_pipe = skinned_pipe.fragment(entry);
        }
        let skinned_pipeline = skinned_pipe.make();

        let initial_skinned_vertex_count = 10000;
        let initial_skinned_index_count = 30000;

        let skinned_vertex_buffer = build::buffer(
            device,
            "Shadow Skinned Vertex Buffer",
            (std::mem::size_of::<SkinnedVertex>() * initial_skinned_vertex_count) as u64,
            wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        let skinned_index_buffer = build::buffer(
            device,
            "Shadow Skinned Index Buffer",
            (std::mem::size_of::<u32>() * initial_skinned_index_count) as u64,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        );

        let point_light_layers = super::types::point_shadow_layers(device);
        let point_light_shadow_count = super::types::point_shadow_slots(device);
        let point_light_cubemap = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Point Light Shadow Cubemap Array"),
            size: nightshade_wgpu::backend::texture_extent(
                device,
                POINT_SHADOW_FACE_SIZE,
                POINT_SHADOW_FACE_SIZE,
                point_light_layers,
            ),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: point_shadow_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let point_light_cubemap_view =
            point_light_cubemap.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Point Light Shadow Cubemap Array View"),
                format: Some(point_shadow_format),
                dimension: Some(wgpu::TextureViewDimension::CubeArray),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(point_light_layers),
                usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
            });

        let mut point_light_face_views = Vec::with_capacity(point_light_layers as usize);
        for light_index in 0..point_light_shadow_count {
            for face_index in 0..POINT_SHADOW_NUM_FACES {
                let layer = (light_index * POINT_SHADOW_NUM_FACES + face_index) as u32;
                let view = point_light_cubemap.create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&format!(
                        "Point Light {} Face {} View",
                        light_index, face_index
                    )),
                    format: Some(point_shadow_format),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: Some(1),
                    base_array_layer: layer,
                    array_layer_count: Some(1),
                    usage: Some(wgpu::TextureUsages::RENDER_ATTACHMENT),
                });
                point_light_face_views.push(view);
            }
        }

        let point_light_depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Point Light Shadow Depth Texture"),
            size: nightshade_wgpu::backend::texture_extent(
                device,
                POINT_SHADOW_FACE_SIZE,
                POINT_SHADOW_FACE_SIZE,
                1,
            ),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let point_light_depth_view =
            point_light_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let point_uniform_bind_group_layout = build::bind_layout(
            device,
            "Point Light Uniform Bind Group Layout",
            &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<PointShadowUniforms>() as u64,
                    ),
                },
                count: None,
            }],
        );

        let total_point_uniforms = point_light_layers as usize;
        let point_light_uniform_buffer = build::uniform(
            device,
            "Point Light Shadow Uniform Buffer",
            (std::mem::size_of::<PointShadowUniforms>() * total_point_uniforms) as u64,
        );

        let point_light_uniform_bind_group = build::binding(
            device,
            "Point Light Shadow Uniform Bind Group",
            &point_uniform_bind_group_layout,
            vec![wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &point_light_uniform_buffer,
                offset: 0,
                size: wgpu::BufferSize::new(std::mem::size_of::<PointShadowUniforms>() as u64),
            })],
        );

        let depth_clear_shader = composer.compile_wgsl(
            device,
            "depth_clear.wgsl",
            nightshade_wgpu_shading::backend::DEPTH_CLEAR,
        );
        let depth_clear_pipeline_layout =
            build::pipeline_layout(device, "Shadow Depth Clear Pipeline Layout", &[]);
        let depth_clear_pipeline = build::Pipe::new(
            device,
            "Spotlight Shadow Depth Clear Pipeline",
            &depth_clear_shader,
        )
        .layout(&depth_clear_pipeline_layout)
        .depth(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Always),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        })
        .vertex("vs_main")
        .make();

        let culling = super::culling::ShadowCulling::new(
            build,
            &uniform_bind_group_layout,
            &point_uniform_bind_group_layout,
            &texture_bind_group_layout,
            material_shader_defs,
            point_shadow_format,
        );

        let mut pass = Self {
            cascades: ShadowCascades {
                data: CascadeData {
                    view_projections: [[[0.0; 4]; 4]; NUM_SHADOW_CASCADES],
                    split_distances: CASCADE_SPLIT_DISTANCES,
                    atlas_offsets: [[0.0, 0.0], [0.5, 0.0], [0.0, 0.5], [0.5, 0.5]],
                    atlas_scale: [0.5, 0.5],
                    padding: [0.0, 0.0],
                },
                uniforms: cascade_uniform_buffers,
                uniform_groups: cascade_uniform_bind_groups,
            },
            transforms: ShadowTransforms {
                uniforms: uniform_buffer,
                uniform_layout: uniform_bind_group_layout,
                buffer: transform_buffer,
                bind_group: transform_bind_group,
                layout: transform_bind_group_layout,
                buffer_size: initial_transform_count,
            },
            materials: ShadowMaterials {
                texture_layout: texture_bind_group_layout,
                bindless_max: material_bindless_max,
                bind_group: None,
                layer_map: HashMap::new(),
                layer_generation: 0,
                buffer: material_buffer,
                buffer_size: material_buffer_size,
                indices: HashMap::new(),
                generation: u64::MAX,
                row_layer_generation: 0,
                class_generation: u64::MAX,
            },
            point_lights: ShadowPointLights {
                cubemap: point_light_cubemap_view,
                publish_pending: true,
                faces: point_light_face_views,
                depth: point_light_depth_view,
                uniforms: point_light_uniform_buffer,
                uniform_group: point_light_uniform_bind_group,
                slots: Vec::new(),
                data: Vec::new(),
                slot_cache: [ShadowSlotCache::default(); MAX_POINT_LIGHT_SHADOWS],
            },
            spotlights: ShadowSpotlights {
                slots: Vec::new(),
                data: Vec::new(),
                atlas_slot_size: 0,
                slot_cache: [ShadowSlotCache::default(); MAX_SPOTLIGHT_SHADOWS],
            },
            skinned_geometry: ShadowSkinnedGeometry {
                vertex: skinned_vertex_buffer,
                index: skinned_index_buffer,
                vertex_size: (std::mem::size_of::<SkinnedVertex>() * initial_skinned_vertex_count)
                    as u64,
                index_size: (std::mem::size_of::<u32>() * initial_skinned_index_count) as u64,
                ids: HashMap::new(),
                vertex_offset: 0,
                index_offset: 0,
                compaction_counter: 0,
                casters: Vec::new(),
            },
            skinned_draw: ShadowSkinnedDraw {
                pipeline: skinned_pipeline,
                objects: skinned_object_buffer,
                objects_size: initial_skinned_object_count,
                indirect: skinned_indirect_buffer,
                indirect_size: initial_skinned_indirect_count,
                group_count: 0,
                cull_view_count: 0,
                occluder_count: 0,
                cull_objects: skinned_cull_objects_buffer,
                cull_objects_size: initial_skinned_cull_objects,
                visible_indices: skinned_visible_indices_buffer,
                visible_indices_size: initial_skinned_visible,
                cull_pipeline: skinned_cull_pipeline,
                cull_layout: skinned_cull_bind_group_layout,
                cull_uniforms: skinned_cull_uniform_buffers,
                cull_groups: skinned_cull_bind_groups,
                bind_group: skinned_bind_group,
                layout: skinned_bind_group_layout,
            },
            skinning: ShadowSkinning {
                pipeline: skinning_compute_pipeline,
                bind_group: skinning_compute_bind_group,
                layout: skinning_compute_bind_group_layout,
                joints: joint_matrices_buffer,
                bone_transforms: bone_transforms_buffer,
                inverse_binds: inverse_bind_matrices_buffer,
                skin_data: skin_data_buffer,
                bone_transforms_size: INITIAL_MAX_TOTAL_BONES,
                inverse_binds_size: initial_joint_matrices_size,
                joints_size: initial_joint_matrices_size,
                skin_data_size: INITIAL_MAX_SKINS,
                bone_source_external: false,
                bone_bound: true,
                joints_to_dispatch: 0,
            },
            occluders: ShadowOccluders {
                rows: Vec::new(),
                transforms: Vec::new(),
                cache: Vec::new(),
                slot_of: HashMap::new(),
                pending_casters: Vec::new(),
                instanced_casters: std::collections::HashSet::new(),
                casters: Vec::new(),
                frame_changes: super::types::ShadowFrameChanges::default(),
                seen_geo_count: 0,
                initialized: false,
                mesh_generation: 0,
            },
            geometry: ShadowGeometry {
                vertex: vertex_buffer,
                index: index_buffer,
                vertex_size: (std::mem::size_of::<ShadowVertex>() * initial_vertex_count) as u64,
                index_size: (std::mem::size_of::<u32>() * initial_index_count) as u64,
                ids: HashMap::new(),
                oversized: std::collections::HashSet::new(),
                vertex_offset: 0,
                index_offset: 0,
                free_vertex: Vec::new(),
                free_index: Vec::new(),
                compaction_counter: 0,
                crowded_at: 0,
            },
            dirty_mesh_names: std::collections::HashSet::new(),
            shadow_scene_dirty: false,
            was_enabled_last_frame: false,
            shadow_view_dirty: Vec::new(),
            depth_clear_pipeline,
            culling,
            buffer_cap: device.limits().max_buffer_size.min(BUFFER_CEILING),
        };

        pass.add_builtin_meshes(device, queue);

        pass
    }
    pub fn add_mesh(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: String,
        vertices: &[Vertex],
        indices: &[u32],
    ) {
        let packed: Vec<ShadowVertex> = vertices.iter().map(ShadowVertex::from).collect();
        let vertex_size = std::mem::size_of_val(packed.as_slice()) as u64;
        let index_size = std::mem::size_of_val(indices) as u64;
        let vertex_offset = take_span(
            &mut self.geometry.free_vertex,
            &mut self.geometry.vertex_offset,
            vertices.len() as u32,
        );
        let index_offset = take_span(
            &mut self.geometry.free_index,
            &mut self.geometry.index_offset,
            indices.len() as u32,
        );

        let required_vertex_size =
            (vertex_offset as u64 * std::mem::size_of::<ShadowVertex>() as u64) + vertex_size;
        let required_index_size =
            (index_offset as u64 * std::mem::size_of::<u32>() as u64) + index_size;

        if required_vertex_size > self.buffer_cap || required_index_size > self.buffer_cap {
            give_span(
                &mut self.geometry.free_vertex,
                vertex_offset,
                vertices.len() as u32,
            );
            give_span(
                &mut self.geometry.free_index,
                index_offset,
                indices.len() as u32,
            );
            self.geometry.oversized.insert(name);
            return;
        }

        if required_vertex_size > self.geometry.vertex_size {
            let new_size = (required_vertex_size * 2)
                .max(self.geometry.vertex_size * 2)
                .min(self.buffer_cap);
            let new_buffer = build::buffer(
                device,
                "Shadow Vertex Buffer (Resized)",
                new_size,
                wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            );

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Vertex Buffer Copy"),
            });
            encoder.copy_buffer_to_buffer(
                &self.geometry.vertex,
                0,
                &new_buffer,
                0,
                self.geometry.vertex_size,
            );
            queue.submit(Some(encoder.finish()));

            self.geometry.vertex = new_buffer;
            self.geometry.vertex_size = new_size;
        }

        if required_index_size > self.geometry.index_size {
            let new_size = (required_index_size * 2)
                .max(self.geometry.index_size * 2)
                .min(self.buffer_cap);
            let new_buffer = build::buffer(
                device,
                "Shadow Index Buffer (Resized)",
                new_size,
                wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            );

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Index Buffer Copy"),
            });
            encoder.copy_buffer_to_buffer(
                &self.geometry.index,
                0,
                &new_buffer,
                0,
                self.geometry.index_size,
            );
            queue.submit(Some(encoder.finish()));

            self.geometry.index = new_buffer;
            self.geometry.index_size = new_size;
        }

        queue.write_buffer(
            &self.geometry.vertex,
            vertex_offset as u64 * std::mem::size_of::<ShadowVertex>() as u64,
            bytemuck::cast_slice(&packed),
        );

        queue.write_buffer(
            &self.geometry.index,
            index_offset as u64 * std::mem::size_of::<u32>() as u64,
            bytemuck::cast_slice(indices),
        );

        self.culling.register_mesh(
            &name,
            indices.len() as u32,
            index_offset,
            vertex_offset as i32,
            vertices,
        );

        self.geometry.ids.insert(
            name,
            (
                vertex_offset,
                vertices.len() as u32,
                index_offset,
                indices.len() as u32,
            ),
        );
    }

    fn add_builtin_meshes(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        use nightshade_render::data::geometry::{
            create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_plane_mesh,
            create_quad_mesh, create_sphere_mesh, create_subdivided_plane_mesh, create_torus_mesh,
        };
        let builtins = [
            ("Cube".to_string(), create_cube_mesh()),
            ("Sphere".to_string(), create_sphere_mesh(1.0, 16)),
            ("Torus".to_string(), create_torus_mesh(1.0, 0.3, 16, 16)),
            ("Plane".to_string(), create_plane_mesh(2.0)),
            (
                "SubdividedPlane".to_string(),
                create_subdivided_plane_mesh(2.0, 20),
            ),
            ("Cylinder".to_string(), create_cylinder_mesh(0.5, 1.0, 16)),
            ("Cone".to_string(), create_cone_mesh(0.5, 1.0, 16)),
            ("Quad".to_string(), create_quad_mesh(1.0)),
        ];
        for (name, mesh) in builtins {
            self.add_mesh(device, queue, name, &mesh.vertices, &mesh.indices);
        }
    }

    pub fn compaction_due(&self) -> bool {
        self.crowded() || self.geometry.compaction_counter + 1 >= COMPACTION_CHECK_INTERVAL_FRAMES
    }

    fn crowded(&self) -> bool {
        let used_vertex_bytes =
            self.geometry.vertex_offset as u64 * std::mem::size_of::<ShadowVertex>() as u64;
        let used_index_bytes =
            self.geometry.index_offset as u64 * std::mem::size_of::<u32>() as u64;
        let near_cap =
            used_vertex_bytes * 2 > self.buffer_cap || used_index_bytes * 2 > self.buffer_cap;
        near_cap && used_vertex_bytes + used_index_bytes > self.geometry.crowded_at
    }

    pub fn note_dirty_meshes(&mut self, names: &std::collections::HashSet<String>) {
        for name in names {
            if self.geometry.ids.contains_key(name.as_str()) {
                self.shadow_scene_dirty = true;
            }
            self.dirty_mesh_names.insert(name.clone());
        }
    }

    pub fn note_frame_changes(
        &mut self,
        frame_state: &nightshade_render::resources::mesh_state::MeshRenderState,
    ) {
        let changes = &mut self.occluders.frame_changes;
        changes
            .added
            .extend(frame_state.entities_added.iter().copied());
        changes
            .removed
            .extend(frame_state.entities_removed.iter().copied());
        changes
            .moved
            .extend(frame_state.transform_dirty.iter().copied());
        changes
            .state_dirty
            .extend(frame_state.object_state_dirty.iter().copied());
        changes
            .material_dirty
            .extend(frame_state.material_dirty.iter().copied());
    }

    pub fn sync_meshes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_cache: &nightshade_render::resources::mesh_cache::MeshCache,
        casting: Option<&std::collections::HashSet<String>>,
        wanted_now: &[String],
        named_casters: bool,
    ) {
        let every = casting.is_some_and(|casting| casting.is_empty()) && !named_casters;
        let compacted = match casting {
            Some(casting) => {
                self.compact_shadow_meshes_if_warranted(device, queue, mesh_cache, casting, every)
            }
            None => {
                self.geometry.compaction_counter += 1;
                false
            }
        };

        let taken: Vec<String> = std::mem::take(&mut self.dirty_mesh_names)
            .into_iter()
            .collect();
        for name in &taken {
            if let Some((vertex_offset, vertex_count, index_offset, index_count)) =
                self.geometry.ids.remove(name)
            {
                give_span(&mut self.geometry.free_vertex, vertex_offset, vertex_count);
                give_span(&mut self.geometry.free_index, index_offset, index_count);
            }
            self.geometry.oversized.remove(name);
        }

        let wanted: Vec<String> = match casting {
            Some(_) if every => mesh_cache_iter(mesh_cache)
                .map(|(name, _)| name.clone())
                .collect(),
            Some(casting) => casting.iter().cloned().collect(),
            None => taken
                .into_iter()
                .chain(wanted_now.iter().cloned())
                .collect(),
        };
        let oversized_before = self.geometry.oversized.len();
        for name in &wanted {
            let Some(mesh) = registry_entry_by_name(&mesh_cache.registry, name) else {
                continue;
            };
            if mesh.skin_data.is_some() {
                continue;
            }

            if self.geometry.ids.contains_key(name.as_str())
                || self.geometry.oversized.contains(name.as_str())
            {
                continue;
            }
            self.add_mesh(device, queue, name.clone(), &mesh.vertices, &mesh.indices);
        }

        let oversized = self.geometry.oversized.len() - oversized_before;
        if oversized > 0 {
            tracing::warn!(
                "{} of {} shadow meshes do not fit the {} MB shadow buffer, so they cast no shadow",
                oversized,
                wanted.len(),
                self.buffer_cap / (1024 * 1024)
            );
        }

        if compacted {
            self.shrink_shadow_buffers(device, queue);
        }
    }

    fn compact_shadow_meshes_if_warranted(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_cache: &nightshade_render::resources::mesh_cache::MeshCache,
        casting: &std::collections::HashSet<String>,
        every: bool,
    ) -> bool {
        let used_vertex_bytes =
            self.geometry.vertex_offset as u64 * std::mem::size_of::<ShadowVertex>() as u64;
        let used_index_bytes =
            self.geometry.index_offset as u64 * std::mem::size_of::<u32>() as u64;

        self.geometry.compaction_counter += 1;
        let timer_elapsed = self.geometry.compaction_counter >= COMPACTION_CHECK_INTERVAL_FRAMES;
        let crowded = self.crowded();
        if !timer_elapsed && !crowded {
            return false;
        }
        if timer_elapsed {
            self.geometry.compaction_counter = 0;
        }

        let mut live_vertex_bytes = 0u64;
        let mut live_index_bytes = 0u64;
        let measure = |mesh: &nightshade_render::data::geometry::Mesh,
                       vertices: &mut u64,
                       indices: &mut u64| {
            if mesh.skin_data.is_some() {
                return;
            }
            *vertices += (mesh.vertices.len() * std::mem::size_of::<ShadowVertex>()) as u64;
            *indices += (mesh.indices.len() * std::mem::size_of::<u32>()) as u64;
        };
        if every {
            for (_, mesh) in mesh_cache_iter(mesh_cache) {
                measure(mesh, &mut live_vertex_bytes, &mut live_index_bytes);
            }
        } else {
            for name in casting {
                let Some(mesh) = registry_entry_by_name(&mesh_cache.registry, name) else {
                    continue;
                };
                measure(mesh, &mut live_vertex_bytes, &mut live_index_bytes);
            }
        }

        let reclaimable = used_vertex_bytes.saturating_sub(live_vertex_bytes)
            + used_index_bytes.saturating_sub(live_index_bytes);
        let utilization = if used_vertex_bytes > 0 {
            live_vertex_bytes as f32 / used_vertex_bytes as f32
        } else {
            1.0
        };

        let frees_room = reclaimable >= COMPACTION_MIN_RECLAIM_BYTES;
        if crowded && !frees_room {
            self.geometry.crowded_at = used_vertex_bytes + used_index_bytes;
        }
        let worth = frees_room && (crowded || utilization < COMPACTION_UTILIZATION_THRESHOLD);
        if !worth {
            return false;
        }

        self.geometry.crowded_at = 0;
        self.geometry.ids.clear();
        self.geometry.oversized.clear();
        self.geometry.free_vertex.clear();
        self.geometry.free_index.clear();
        self.geometry.vertex_offset = 0;
        self.geometry.index_offset = 0;
        self.culling.clear_mesh_table();
        self.add_builtin_meshes(device, queue);
        true
    }

    fn shrink_shadow_buffers(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let used_vertex_bytes =
            self.geometry.vertex_offset as u64 * std::mem::size_of::<ShadowVertex>() as u64;
        let used_index_bytes =
            self.geometry.index_offset as u64 * std::mem::size_of::<u32>() as u64;

        let new_vertex_size = (used_vertex_bytes * 2).min(self.buffer_cap);
        if used_vertex_bytes > 0 && new_vertex_size < self.geometry.vertex_size {
            let new_buffer = build::buffer(
                device,
                "Shadow Vertex Buffer (Compacted)",
                new_vertex_size,
                wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            );
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Vertex Buffer Compaction"),
            });
            encoder.copy_buffer_to_buffer(
                &self.geometry.vertex,
                0,
                &new_buffer,
                0,
                used_vertex_bytes,
            );
            queue.submit(Some(encoder.finish()));
            self.geometry.vertex = new_buffer;
            self.geometry.vertex_size = new_vertex_size;
        }

        let new_index_size = (used_index_bytes * 2).min(self.buffer_cap);
        if used_index_bytes > 0 && new_index_size < self.geometry.index_size {
            let new_buffer = build::buffer(
                device,
                "Shadow Index Buffer (Compacted)",
                new_index_size,
                wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            );
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Index Buffer Compaction"),
            });
            encoder.copy_buffer_to_buffer(
                &self.geometry.index,
                0,
                &new_buffer,
                0,
                used_index_bytes,
            );
            queue.submit(Some(encoder.finish()));
            self.geometry.index = new_buffer;
            self.geometry.index_size = new_index_size;
        }
    }

    pub fn sync_skinned_meshes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_cache: &nightshade_render::resources::mesh_cache::MeshCache,
    ) {
        let compacted = self.compact_skinned_shadow_meshes_if_warranted(mesh_cache);

        for (name, mesh) in mesh_cache_iter(mesh_cache) {
            if mesh.skin_data.is_none() {
                continue;
            }

            if self.skinned_geometry.ids.contains_key(name.as_str()) {
                continue;
            }

            let skin_data = mesh.skin_data.as_ref().unwrap();
            let skinned_vertices = &skin_data.skinned_vertices;

            let vertex_size =
                (skinned_vertices.len() * std::mem::size_of::<SkinnedVertex>()) as u64;
            let index_size = (mesh.indices.len() * std::mem::size_of::<u32>()) as u64;

            let required_vertex_size = (self.skinned_geometry.vertex_offset as u64
                * std::mem::size_of::<SkinnedVertex>() as u64)
                + vertex_size;
            let required_index_size = (self.skinned_geometry.index_offset as u64
                * std::mem::size_of::<u32>() as u64)
                + index_size;

            if required_vertex_size > self.buffer_cap || required_index_size > self.buffer_cap {
                tracing::warn!(
                    "Shadow skinned mesh {} exceeds maximum buffer size, skipping shadow rendering",
                    name
                );
                self.geometry.oversized.insert(name.clone());
                continue;
            }

            if required_vertex_size > self.skinned_geometry.vertex_size {
                let new_size = (required_vertex_size * 2)
                    .max(self.skinned_geometry.vertex_size * 2)
                    .min(self.buffer_cap);
                let new_buffer = build::buffer(
                    device,
                    "Shadow Skinned Vertex Buffer (Resized)",
                    new_size,
                    wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                );

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Shadow Skinned Vertex Buffer Copy"),
                });
                encoder.copy_buffer_to_buffer(
                    &self.skinned_geometry.vertex,
                    0,
                    &new_buffer,
                    0,
                    self.skinned_geometry.vertex_size,
                );
                queue.submit(Some(encoder.finish()));

                self.skinned_geometry.vertex = new_buffer;
                self.skinned_geometry.vertex_size = new_size;
            }

            if required_index_size > self.skinned_geometry.index_size {
                let new_size = (required_index_size * 2)
                    .max(self.skinned_geometry.index_size * 2)
                    .min(self.buffer_cap);
                let new_buffer = build::buffer(
                    device,
                    "Shadow Skinned Index Buffer (Resized)",
                    new_size,
                    wgpu::BufferUsages::INDEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                );

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Shadow Skinned Index Buffer Copy"),
                });
                encoder.copy_buffer_to_buffer(
                    &self.skinned_geometry.index,
                    0,
                    &new_buffer,
                    0,
                    self.skinned_geometry.index_size,
                );
                queue.submit(Some(encoder.finish()));

                self.skinned_geometry.index = new_buffer;
                self.skinned_geometry.index_size = new_size;
            }

            queue.write_buffer(
                &self.skinned_geometry.vertex,
                self.skinned_geometry.vertex_offset as u64
                    * std::mem::size_of::<SkinnedVertex>() as u64,
                bytemuck::cast_slice(skinned_vertices),
            );

            queue.write_buffer(
                &self.skinned_geometry.index,
                self.skinned_geometry.index_offset as u64 * std::mem::size_of::<u32>() as u64,
                bytemuck::cast_slice(&mesh.indices),
            );

            self.skinned_geometry.ids.insert(
                name.clone(),
                (
                    self.skinned_geometry.vertex_offset,
                    skinned_vertices.len() as u32,
                    self.skinned_geometry.index_offset,
                    mesh.indices.len() as u32,
                ),
            );

            self.skinned_geometry.vertex_offset += skinned_vertices.len() as u32;
            self.skinned_geometry.index_offset += mesh.indices.len() as u32;
        }

        if compacted {
            self.shrink_skinned_shadow_buffers(device, queue);
        }
    }

    fn compact_skinned_shadow_meshes_if_warranted(
        &mut self,
        mesh_cache: &nightshade_render::resources::mesh_cache::MeshCache,
    ) -> bool {
        let used_vertex_bytes = self.skinned_geometry.vertex_offset as u64
            * std::mem::size_of::<SkinnedVertex>() as u64;
        let used_index_bytes =
            self.skinned_geometry.index_offset as u64 * std::mem::size_of::<u32>() as u64;

        self.skinned_geometry.compaction_counter += 1;
        let timer_elapsed =
            self.skinned_geometry.compaction_counter >= COMPACTION_CHECK_INTERVAL_FRAMES;
        let near_cap =
            used_vertex_bytes * 2 > self.buffer_cap || used_index_bytes * 2 > self.buffer_cap;
        if !timer_elapsed && !near_cap {
            return false;
        }
        if timer_elapsed {
            self.skinned_geometry.compaction_counter = 0;
        }

        let mut live_vertex_bytes = 0u64;
        let mut live_index_bytes = 0u64;
        for (_, mesh) in mesh_cache_iter(mesh_cache) {
            let Some(skin_data) = mesh.skin_data.as_ref() else {
                continue;
            };
            live_vertex_bytes +=
                (skin_data.skinned_vertices.len() * std::mem::size_of::<SkinnedVertex>()) as u64;
            live_index_bytes += (mesh.indices.len() * std::mem::size_of::<u32>()) as u64;
        }

        let reclaimable = used_vertex_bytes.saturating_sub(live_vertex_bytes)
            + used_index_bytes.saturating_sub(live_index_bytes);
        let utilization = if used_vertex_bytes > 0 {
            live_vertex_bytes as f32 / used_vertex_bytes as f32
        } else {
            1.0
        };

        if reclaimable < COMPACTION_MIN_RECLAIM_BYTES
            || utilization >= COMPACTION_UTILIZATION_THRESHOLD
        {
            return false;
        }

        self.skinned_geometry.ids.clear();
        self.skinned_geometry.vertex_offset = 0;
        self.skinned_geometry.index_offset = 0;
        true
    }

    fn shrink_skinned_shadow_buffers(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let used_vertex_bytes = self.skinned_geometry.vertex_offset as u64
            * std::mem::size_of::<SkinnedVertex>() as u64;
        let used_index_bytes =
            self.skinned_geometry.index_offset as u64 * std::mem::size_of::<u32>() as u64;

        let new_vertex_size = (used_vertex_bytes * 2).min(self.buffer_cap);
        if used_vertex_bytes > 0 && new_vertex_size < self.skinned_geometry.vertex_size {
            let new_buffer = build::buffer(
                device,
                "Shadow Skinned Vertex Buffer (Compacted)",
                new_vertex_size,
                wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            );
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Skinned Vertex Buffer Compaction"),
            });
            encoder.copy_buffer_to_buffer(
                &self.skinned_geometry.vertex,
                0,
                &new_buffer,
                0,
                used_vertex_bytes,
            );
            queue.submit(Some(encoder.finish()));
            self.skinned_geometry.vertex = new_buffer;
            self.skinned_geometry.vertex_size = new_vertex_size;
        }

        let new_index_size = (used_index_bytes * 2).min(self.buffer_cap);
        if used_index_bytes > 0 && new_index_size < self.skinned_geometry.index_size {
            let new_buffer = build::buffer(
                device,
                "Shadow Skinned Index Buffer (Compacted)",
                new_index_size,
                wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            );
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Skinned Index Buffer Compaction"),
            });
            encoder.copy_buffer_to_buffer(
                &self.skinned_geometry.index,
                0,
                &new_buffer,
                0,
                used_index_bytes,
            );
            queue.submit(Some(encoder.finish()));
            self.skinned_geometry.index = new_buffer;
            self.skinned_geometry.index_size = new_index_size;
        }
    }

    pub fn rebuild_skinned_bind_group(&mut self, device: &wgpu::Device) {
        self.skinned_draw.bind_group = buffer_bind_group(
            device,
            "Shadow Skinned Bind Group",
            &self.skinned_draw.layout,
            &[
                &self.skinned_draw.objects,
                &self.skinning.joints,
                &self.skinned_draw.visible_indices,
                &self.materials.buffer,
            ],
        );
    }

    pub fn rebuild_skinned_cull_bind_groups(&mut self, device: &wgpu::Device) {
        self.skinned_draw.cull_groups = self
            .skinned_draw
            .cull_uniforms
            .iter()
            .map(|uniform_buffer| {
                buffer_bind_group(
                    device,
                    "Shadow Skinned Cull Bind Group",
                    &self.skinned_draw.cull_layout,
                    &[
                        &self.skinned_draw.cull_objects,
                        &self.skinned_draw.indirect,
                        &self.skinned_draw.visible_indices,
                        uniform_buffer,
                    ],
                )
            })
            .collect();
    }

    pub fn set_bone_source(&mut self, buffer: wgpu::Buffer) {
        self.skinning.bone_transforms = buffer;
        self.skinning.bone_source_external = true;
    }

    pub fn rebuild_skinning_compute_bind_group(&mut self, device: &wgpu::Device) {
        self.skinning.bind_group = buffer_bind_group(
            device,
            "Shadow Skinning Compute Bind Group",
            &self.skinning.layout,
            &[
                &self.skinning.bone_transforms,
                &self.skinning.inverse_binds,
                &self.skinning.skin_data,
                &self.skinning.joints,
            ],
        );
    }
}

impl ShadowDepthPass {
    pub fn apply_material_textures(
        &mut self,
        device: &wgpu::Device,
        arrays: &MaterialTextureArrays,
    ) {
        if self.materials.bindless_max.is_some() && arrays.is_bindless() {
            let views = arrays.bindless_view_refs();
            let samplers = arrays.samplers();
            self.materials.bind_group = Some(material_gpu::bindless_material_bind_group(
                device,
                &self.materials.texture_layout,
                &views,
                &samplers,
                "Shadow Material Bindless Bind Group",
            ));
            return;
        }
        self.materials.bind_group = Some(material_gpu::array_material_bind_group(
            device,
            &self.materials.texture_layout,
            arrays.srgb_view(),
            arrays.linear_view(),
            &arrays.samplers(),
            "Shadow Material Texture Bind Group",
        ));
    }

    pub fn add_material_layer_mapping(&mut self, texture: TextureId, layer: MaterialTextureLayer) {
        self.materials.layer_map.insert(texture, layer);
        self.materials.layer_generation += 1;
    }

    pub(super) fn shadow_material_index(
        &self,
        configs: &RenderInputs,
        entity: nightshade::ecs::Entity,
    ) -> u32 {
        configs
            .scene_world
            .get::<nightshade_render::components::render_world::MaterialId>(entity)
            .and_then(|material| self.materials.indices.get(&material.0))
            .copied()
            .unwrap_or(super::types::NO_SHADOW_MATERIAL)
    }

    pub(super) fn rebuild_shadow_materials(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        materials: &RenderMaterials,
    ) {
        self.materials.indices.clear();
        let mut data: Vec<MaterialData> = Vec::new();
        for (index, entry) in materials.entries.iter().enumerate() {
            if !matches!(
                entry.material.alpha_mode,
                AlphaMode::Mask | AlphaMode::Dither
            ) {
                continue;
            }
            self.materials
                .indices
                .insert(index as u32 + 1, data.len() as u32);
            data.push(convert_material_to_gpu_data(
                &entry.material,
                &entry.texture_ids,
                &self.materials.layer_map,
            ));
        }
        self.materials.generation = materials.generation;
        self.materials.row_layer_generation = self.materials.layer_generation;
        if data.is_empty() {
            return;
        }
        if grow_storage_buffer::<MaterialData>(
            device,
            &mut self.materials.buffer,
            &mut self.materials.buffer_size,
            data.len(),
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "Shadow Material Buffer (Resized)",
        ) {
            self.rebuild_skinned_bind_group(device);
        }
        queue.write_buffer(&self.materials.buffer, 0, bytemuck::cast_slice(&data));
    }
}
