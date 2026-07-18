use crate::geometry::{SkinnedVertex, Vertex};
use crate::mesh_cache::mesh_cache_iter;
use crate::skinning::GpuSkinData;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

const MAX_BUFFER_SIZE: u64 = 256 * 1024 * 1024;

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

pub struct ShadowDepthPass {
    pub(super) uniform_buffer: wgpu::Buffer,
    pub(super) uniform_bind_group_layout: wgpu::BindGroupLayout,

    pub(super) transform_buffer: wgpu::Buffer,
    pub(super) transform_bind_group: wgpu::BindGroup,
    pub(super) transform_bind_group_layout: wgpu::BindGroupLayout,

    pub(super) transform_buffer_size: usize,

    pub(super) vertex_buffer: wgpu::Buffer,
    pub(super) vertex_buffer_generation: u64,
    pub(super) index_buffer: wgpu::Buffer,
    pub(super) vertex_buffer_size: u64,
    pub(super) index_buffer_size: u64,

    pub(super) meshes: HashMap<String, (u32, u32, u32, u32)>,

    pub(super) current_vertex_offset: u32,
    pub(super) current_index_offset: u32,
    pub(super) compaction_counter: u64,

    pub(super) shadow_caster_entities: Vec<freecs::Entity>,

    pub(super) occluders: Vec<super::types::ShadowOccluder>,
    pub(super) occluder_transforms: Vec<[[f32; 4]; 4]>,
    pub(super) occluder_cache: Vec<super::node::OccluderCacheEntry>,
    pub(super) all_caster_entities: Vec<freecs::Entity>,
    pub(super) last_visible_signature: u64,
    pub(super) occluders_initialized: bool,
    pub shadow_scene_dirty: bool,

    pub(super) skinned_pipeline: wgpu::RenderPipeline,
    pub(super) skinned_object_buffer: wgpu::Buffer,
    pub(super) skinned_object_buffer_size: usize,
    pub(super) skinned_indirect_buffer: wgpu::Buffer,
    pub(super) skinned_indirect_buffer_size: usize,
    pub(super) skinned_group_count: usize,
    pub(super) skinned_cull_view_count: usize,
    pub(super) skinned_occluder_count: usize,
    pub(super) skinned_cull_objects_buffer: wgpu::Buffer,
    pub(super) skinned_cull_objects_buffer_size: usize,
    pub(super) skinned_visible_indices_buffer: wgpu::Buffer,
    pub(super) skinned_visible_indices_buffer_size: usize,
    pub(super) skinned_cull_pipeline: wgpu::ComputePipeline,
    pub(super) skinned_cull_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) skinned_cull_uniform_buffers: Vec<wgpu::Buffer>,
    pub(super) skinned_cull_bind_groups: Vec<wgpu::BindGroup>,
    pub(super) skinned_bind_group: wgpu::BindGroup,
    pub(super) skinned_bind_group_layout: wgpu::BindGroupLayout,
    pub(super) joint_matrices_buffer: wgpu::Buffer,
    pub(super) bone_transforms_buffer: wgpu::Buffer,
    pub(super) inverse_bind_matrices_buffer: wgpu::Buffer,
    pub(super) skin_data_buffer: wgpu::Buffer,
    pub(super) bone_transforms_buffer_size: usize,
    pub(super) inverse_bind_matrices_buffer_size: usize,
    pub(super) joint_matrices_buffer_size: usize,
    pub(super) skin_data_buffer_size: usize,
    pub(super) skinning_compute_pipeline: wgpu::ComputePipeline,
    pub(super) skinning_compute_bind_group: wgpu::BindGroup,
    pub(super) skinning_compute_bind_group_layout: wgpu::BindGroupLayout,

    pub(super) skinned_vertex_buffer: wgpu::Buffer,
    pub(super) skinned_index_buffer: wgpu::Buffer,
    pub(super) skinned_vertex_buffer_size: u64,
    pub(super) skinned_index_buffer_size: u64,
    pub(super) skinned_meshes: HashMap<String, (u32, u32, u32, u32)>,
    pub(super) current_skinned_vertex_offset: u32,
    pub(super) current_skinned_index_offset: u32,
    pub(super) skinned_compaction_counter: u64,
    pub(super) skinned_shadow_caster_entities: Vec<freecs::Entity>,
    pub(super) total_joints_to_dispatch: u32,

    pub(super) spotlight_shadow_slots: Vec<SpotlightShadowSlot>,
    pub(super) spotlight_shadow_data: Vec<SpotlightShadowData>,
    pub(super) spotlight_atlas_slot_size: u32,

    pub(super) spotlight_slot_cache: [ShadowSlotCache; MAX_SPOTLIGHT_SHADOWS],
    pub(super) point_slot_cache: [ShadowSlotCache; MAX_POINT_LIGHT_SHADOWS],
    pub(super) was_enabled_last_frame: bool,
    pub(super) shadow_view_dirty: Vec<bool>,
    pub(super) depth_clear_pipeline: wgpu::RenderPipeline,

    pub point_light_cubemap: wgpu::Texture,
    pub(super) point_light_cubemap_view: wgpu::TextureView,
    pub(super) point_light_face_views: Vec<wgpu::TextureView>,
    pub point_light_depth_texture: wgpu::Texture,
    pub(super) point_light_depth_view: wgpu::TextureView,
    pub(super) point_light_uniform_buffer: wgpu::Buffer,
    pub(super) point_light_uniform_bind_group: wgpu::BindGroup,
    pub(super) point_light_shadow_slots: Vec<PointLightShadowSlot>,
    pub(super) point_light_shadow_data: Vec<PointLightShadowData>,

    pub(super) cascade_data: CascadeData,
    pub(super) cascade_uniform_buffers: [wgpu::Buffer; NUM_SHADOW_CASCADES],
    pub(super) cascade_uniform_bind_groups: [wgpu::BindGroup; NUM_SHADOW_CASCADES],

    pub(super) culling: super::culling::ShadowCulling,
}

/// Builds a bind group whose entries are storage/uniform buffers bound in
/// sequence starting at binding 0, each via `as_entire_binding`.
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
    pub async fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Uniform Buffer"),
            size: std::mem::size_of::<ShadowUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let initial_cascade_uniforms = ShadowUniforms {
            light_view_projection: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            _padding: [[0.0; 4]; 4],
            _padding2: [[0.0; 4]; 4],
            _padding3: [[0.0; 4]; 4],
            _padding4: [0.0; 4],
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
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("Cascade {} Uniform Bind Group", index)),
                    layout: &uniform_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: cascade_uniform_buffers[index].as_entire_binding(),
                    }],
                })
            });

        let initial_transform_count = 1000;
        let transform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Transform Buffer"),
            size: (std::mem::size_of::<[[f32; 4]; 4]>() * initial_transform_count) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let transform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Transform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Transform Bind Group"),
            layout: &transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
        });

        let initial_vertex_count = 10000;
        let initial_index_count = 30000;

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Vertex Buffer"),
            size: (std::mem::size_of::<Vertex>() * initial_vertex_count) as u64,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Index Buffer"),
            size: (std::mem::size_of::<u32>() * initial_index_count) as u64,
            usage: wgpu::BufferUsages::INDEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let bone_transforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Bone Transforms Buffer"),
            size: (std::mem::size_of::<[[f32; 4]; 4]>() * INITIAL_MAX_TOTAL_BONES) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let inverse_bind_matrices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Inverse Bind Matrices Buffer"),
            size: (std::mem::size_of::<[[f32; 4]; 4]>() * MAX_JOINTS_PER_SKIN * INITIAL_MAX_SKINS)
                as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let joint_matrices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Joint Matrices Buffer"),
            size: (std::mem::size_of::<[[f32; 4]; 4]>() * MAX_JOINTS_PER_SKIN * INITIAL_MAX_SKINS)
                as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let skin_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Skin Data Buffer"),
            size: (std::mem::size_of::<GpuSkinData>() * INITIAL_MAX_SKINS) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let skinning_compute_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "skinning_compute.wgsl",
            include_str!("../../shaders/skinning_compute.wgsl"),
        );

        let skinning_compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Skinning Compute Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

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

        let skinning_compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Skinning Compute Pipeline Layout"),
                bind_group_layouts: &[Some(&skinning_compute_bind_group_layout)],
                immediate_size: 0,
            });

        let skinning_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Shadow Skinning Compute Pipeline"),
                layout: Some(&skinning_compute_pipeline_layout),
                module: &skinning_compute_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        crate::wgpu::stagger::yield_to_event_loop(queue).await;

        let initial_skinned_object_count = 100;
        let skinned_object_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Skinned Object Buffer"),
            size: (std::mem::size_of::<SkinnedShadowObjectData>() * initial_skinned_object_count)
                as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let initial_skinned_indirect_count = 32;
        let skinned_indirect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Skinned Indirect Buffer"),
            size: (std::mem::size_of::<super::types::ShadowDrawIndexedIndirect>()
                * initial_skinned_indirect_count) as u64,
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let initial_skinned_cull_objects = 100;
        let skinned_cull_objects_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Skinned Cull Objects Buffer"),
            size: (std::mem::size_of::<super::types::SkinnedShadowCullObject>()
                * initial_skinned_cull_objects) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let initial_skinned_visible = 100;
        let skinned_visible_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Skinned Visible Indices Buffer"),
            size: (std::mem::size_of::<u32>() * initial_skinned_visible) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

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
        let skinned_cull_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Skinned Cull Bind Group Layout"),
                entries: &[
                    skinned_cull_storage_entry(0, true),
                    skinned_cull_storage_entry(1, false),
                    skinned_cull_storage_entry(2, false),
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let skinned_cull_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "skinned_shadow_cull.wgsl",
            include_str!("../../shaders/skinned_shadow_cull.wgsl"),
        );
        let skinned_cull_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Skinned Cull Pipeline Layout"),
                bind_group_layouts: &[Some(&skinned_cull_bind_group_layout)],
                immediate_size: 0,
            });
        let skinned_cull_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Shadow Skinned Cull Pipeline"),
                layout: Some(&skinned_cull_pipeline_layout),
                module: &skinned_cull_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        crate::wgpu::stagger::yield_to_event_loop(queue).await;
        let skinned_cull_uniform_buffers: Vec<wgpu::Buffer> = (0..SKINNED_CULL_MAX_VIEWS)
            .map(|view| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("Shadow Skinned Cull Uniform Buffer {view}")),
                    size: std::mem::size_of::<super::types::ShadowCullUniforms>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
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

        let skinned_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Skinned Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let skinned_bind_group = buffer_bind_group(
            device,
            "Shadow Skinned Bind Group",
            &skinned_bind_group_layout,
            &[
                &skinned_object_buffer,
                &joint_matrices_buffer,
                &skinned_visible_indices_buffer,
            ],
        );

        let skinned_shadow_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "skinned_shadow_depth.wgsl",
            include_str!("../../shaders/skinned_shadow_depth.wgsl"),
        );

        let skinned_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Skinned Shadow Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&uniform_bind_group_layout),
                    Some(&skinned_bind_group_layout),
                ],
                immediate_size: 0,
            });

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

        let skinned_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Skinned Shadow Depth Pipeline"),
            layout: Some(&skinned_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &skinned_shadow_shader,
                entry_point: Some("vs_main"),
                buffers: &[skinned_vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &skinned_shadow_shader,
                entry_point: Some("fs_main"),
                targets: &[],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        crate::wgpu::stagger::yield_to_event_loop(queue).await;

        let initial_skinned_vertex_count = 10000;
        let initial_skinned_index_count = 30000;

        let skinned_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Skinned Vertex Buffer"),
            size: (std::mem::size_of::<SkinnedVertex>() * initial_skinned_vertex_count) as u64,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let skinned_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow Skinned Index Buffer"),
            size: (std::mem::size_of::<u32>() * initial_skinned_index_count) as u64,
            usage: wgpu::BufferUsages::INDEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let point_light_cubemap = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Point Light Shadow Cubemap Array"),
            size: wgpu::Extent3d {
                width: POINT_SHADOW_FACE_SIZE,
                height: POINT_SHADOW_FACE_SIZE,
                depth_or_array_layers: (POINT_SHADOW_NUM_FACES * MAX_POINT_LIGHT_SHADOWS) as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let point_light_cubemap_view =
            point_light_cubemap.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Point Light Shadow Cubemap Array View"),
                format: Some(wgpu::TextureFormat::R32Float),
                dimension: Some(wgpu::TextureViewDimension::CubeArray),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some((POINT_SHADOW_NUM_FACES * MAX_POINT_LIGHT_SHADOWS) as u32),
                usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
            });

        let mut point_light_face_views =
            Vec::with_capacity(POINT_SHADOW_NUM_FACES * MAX_POINT_LIGHT_SHADOWS);
        for light_index in 0..MAX_POINT_LIGHT_SHADOWS {
            for face_index in 0..POINT_SHADOW_NUM_FACES {
                let layer = (light_index * POINT_SHADOW_NUM_FACES + face_index) as u32;
                let view = point_light_cubemap.create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&format!(
                        "Point Light {} Face {} View",
                        light_index, face_index
                    )),
                    format: Some(wgpu::TextureFormat::R32Float),
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
            size: wgpu::Extent3d {
                width: POINT_SHADOW_FACE_SIZE,
                height: POINT_SHADOW_FACE_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let point_light_depth_view =
            point_light_depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let point_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Point Light Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                            PointShadowUniforms,
                        >() as u64),
                    },
                    count: None,
                }],
            });

        let total_point_uniforms = MAX_POINT_LIGHT_SHADOWS * POINT_SHADOW_NUM_FACES;
        let point_light_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Point Light Shadow Uniform Buffer"),
            size: (std::mem::size_of::<PointShadowUniforms>() * total_point_uniforms) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let point_light_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Point Light Shadow Uniform Bind Group"),
            layout: &point_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &point_light_uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(std::mem::size_of::<PointShadowUniforms>() as u64),
                }),
            }],
        });

        let depth_clear_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "depth_clear.wgsl",
            include_str!("../../shaders/depth_clear.wgsl"),
        );
        let depth_clear_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Shadow Depth Clear Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        let depth_clear_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Spotlight Shadow Depth Clear Pipeline"),
            layout: Some(&depth_clear_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &depth_clear_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        crate::wgpu::stagger::yield_to_event_loop(queue).await;

        let culling = super::culling::ShadowCulling::new(
            device,
            &uniform_bind_group_layout,
            &point_uniform_bind_group_layout,
        );

        let mut pass = Self {
            uniform_buffer,
            uniform_bind_group_layout,
            transform_buffer,
            transform_bind_group,
            transform_bind_group_layout,
            transform_buffer_size: initial_transform_count,
            vertex_buffer,
            vertex_buffer_generation: 0,
            index_buffer,
            vertex_buffer_size: (std::mem::size_of::<Vertex>() * initial_vertex_count) as u64,
            index_buffer_size: (std::mem::size_of::<u32>() * initial_index_count) as u64,
            meshes: HashMap::new(),
            current_vertex_offset: 0,
            current_index_offset: 0,
            compaction_counter: 0,
            shadow_caster_entities: Vec::new(),
            occluders: Vec::new(),
            occluder_transforms: Vec::new(),
            occluder_cache: Vec::new(),
            all_caster_entities: Vec::new(),
            last_visible_signature: 0,
            occluders_initialized: false,
            shadow_scene_dirty: false,
            skinned_pipeline,
            skinned_object_buffer,
            skinned_object_buffer_size: initial_skinned_object_count,
            skinned_indirect_buffer,
            skinned_indirect_buffer_size: initial_skinned_indirect_count,
            skinned_group_count: 0,
            skinned_cull_view_count: 0,
            skinned_occluder_count: 0,
            skinned_cull_objects_buffer,
            skinned_cull_objects_buffer_size: initial_skinned_cull_objects,
            skinned_visible_indices_buffer,
            skinned_visible_indices_buffer_size: initial_skinned_visible,
            skinned_cull_pipeline,
            skinned_cull_bind_group_layout,
            skinned_cull_uniform_buffers,
            skinned_cull_bind_groups,
            skinned_bind_group,
            skinned_bind_group_layout,
            joint_matrices_buffer,
            bone_transforms_buffer,
            inverse_bind_matrices_buffer,
            skin_data_buffer,
            bone_transforms_buffer_size: INITIAL_MAX_TOTAL_BONES,
            inverse_bind_matrices_buffer_size: initial_joint_matrices_size,
            joint_matrices_buffer_size: initial_joint_matrices_size,
            skin_data_buffer_size: INITIAL_MAX_SKINS,
            skinning_compute_pipeline,
            skinning_compute_bind_group,
            skinning_compute_bind_group_layout,
            skinned_vertex_buffer,
            skinned_index_buffer,
            skinned_vertex_buffer_size: (std::mem::size_of::<SkinnedVertex>()
                * initial_skinned_vertex_count) as u64,
            skinned_index_buffer_size: (std::mem::size_of::<u32>() * initial_skinned_index_count)
                as u64,
            skinned_meshes: HashMap::new(),
            current_skinned_vertex_offset: 0,
            current_skinned_index_offset: 0,
            skinned_compaction_counter: 0,
            skinned_shadow_caster_entities: Vec::new(),
            total_joints_to_dispatch: 0,
            spotlight_shadow_slots: Vec::new(),
            spotlight_shadow_data: Vec::new(),
            spotlight_atlas_slot_size: 0,
            spotlight_slot_cache: [ShadowSlotCache::default(); MAX_SPOTLIGHT_SHADOWS],
            point_slot_cache: [ShadowSlotCache::default(); MAX_POINT_LIGHT_SHADOWS],
            was_enabled_last_frame: false,
            shadow_view_dirty: Vec::new(),
            depth_clear_pipeline,
            point_light_cubemap,
            point_light_cubemap_view,
            point_light_face_views,
            point_light_depth_texture,
            point_light_depth_view,
            point_light_uniform_buffer,
            point_light_uniform_bind_group,
            point_light_shadow_slots: Vec::new(),
            point_light_shadow_data: Vec::new(),
            cascade_data: CascadeData {
                view_projections: [[[0.0; 4]; 4]; NUM_SHADOW_CASCADES],
                split_distances: CASCADE_SPLIT_DISTANCES,
                atlas_offsets: [[0.0, 0.0], [0.5, 0.0], [0.0, 0.5], [0.5, 0.5]],
                atlas_scale: [0.5, 0.5],
                _padding: [0.0, 0.0],
            },
            cascade_uniform_buffers,
            cascade_uniform_bind_groups,
            culling,
        };

        pass.add_builtin_meshes(device, queue);

        pass
    }

    /// Returns a write target for compute passes that deform a registered
    /// mesh in place: a clone of the shadow vertex buffer, the buffer's
    /// generation (bumped whenever the buffer is recreated, so callers know
    /// to rebuild bind groups), and the mesh's vertex offset in elements.
    pub fn vertex_write_target(&self, name: &str) -> Option<(wgpu::Buffer, u64, u32)> {
        let &(vertex_offset, vertex_count, _, _) = self.meshes.get(name)?;
        if vertex_count == 0 {
            return None;
        }
        Some((
            self.vertex_buffer.clone(),
            self.vertex_buffer_generation,
            vertex_offset,
        ))
    }

    pub fn add_mesh(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: String,
        vertices: &[Vertex],
        indices: &[u32],
    ) {
        let vertex_size = std::mem::size_of_val(vertices) as u64;
        let index_size = std::mem::size_of_val(indices) as u64;

        let required_vertex_size = (self.current_vertex_offset as u64
            * std::mem::size_of::<Vertex>() as u64)
            + vertex_size;
        let required_index_size =
            (self.current_index_offset as u64 * std::mem::size_of::<u32>() as u64) + index_size;

        if required_vertex_size > MAX_BUFFER_SIZE || required_index_size > MAX_BUFFER_SIZE {
            tracing::warn!(
                "Shadow mesh {} exceeds maximum buffer size, skipping shadow rendering",
                name
            );
            return;
        }

        if required_vertex_size > self.vertex_buffer_size {
            let new_size = (required_vertex_size * 2)
                .max(self.vertex_buffer_size * 2)
                .min(MAX_BUFFER_SIZE);
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Vertex Buffer (Resized)"),
                size: new_size,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Vertex Buffer Copy"),
            });
            encoder.copy_buffer_to_buffer(
                &self.vertex_buffer,
                0,
                &new_buffer,
                0,
                self.vertex_buffer_size,
            );
            queue.submit(Some(encoder.finish()));

            self.vertex_buffer = new_buffer;
            self.vertex_buffer_size = new_size;
            self.vertex_buffer_generation += 1;
        }

        if required_index_size > self.index_buffer_size {
            let new_size = (required_index_size * 2)
                .max(self.index_buffer_size * 2)
                .min(MAX_BUFFER_SIZE);
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Index Buffer (Resized)"),
                size: new_size,
                usage: wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Index Buffer Copy"),
            });
            encoder.copy_buffer_to_buffer(
                &self.index_buffer,
                0,
                &new_buffer,
                0,
                self.index_buffer_size,
            );
            queue.submit(Some(encoder.finish()));

            self.index_buffer = new_buffer;
            self.index_buffer_size = new_size;
        }

        queue.write_buffer(
            &self.vertex_buffer,
            self.current_vertex_offset as u64 * std::mem::size_of::<Vertex>() as u64,
            bytemuck::cast_slice(vertices),
        );

        queue.write_buffer(
            &self.index_buffer,
            self.current_index_offset as u64 * std::mem::size_of::<u32>() as u64,
            bytemuck::cast_slice(indices),
        );

        self.culling.register_mesh(
            &name,
            indices.len() as u32,
            self.current_index_offset,
            self.current_vertex_offset as i32,
            vertices,
        );

        self.meshes.insert(
            name,
            (
                self.current_vertex_offset,
                vertices.len() as u32,
                self.current_index_offset,
                indices.len() as u32,
            ),
        );

        self.current_vertex_offset += vertices.len() as u32;
        self.current_index_offset += indices.len() as u32;
    }

    /// Registers the built-in primitive meshes that ship with the engine. These
    /// are not stored in the mesh cache, so they must be re-added by hand after a
    /// compaction rebuild as well as at construction.
    fn add_builtin_meshes(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        use crate::geometry::{
            create_cone_mesh, create_cube_mesh, create_cylinder_mesh, create_plane_mesh,
            create_sphere_mesh, create_subdivided_plane_mesh, create_torus_mesh,
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
        ];
        for (name, mesh) in builtins {
            self.add_mesh(device, queue, name, &mesh.vertices, &mesh.indices);
        }
    }

    pub fn sync_meshes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_cache: &crate::mesh_cache::MeshCache,
    ) {
        let compacted = self.compact_shadow_meshes_if_warranted(device, queue, mesh_cache);

        for (name, mesh) in mesh_cache_iter(mesh_cache) {
            if mesh.skin_data.is_some() {
                continue;
            }

            if !self.meshes.contains_key(name.as_str()) {
                let vertex_size = (mesh.vertices.len() * std::mem::size_of::<Vertex>()) as u64;
                let index_size = (mesh.indices.len() * std::mem::size_of::<u32>()) as u64;

                let required_vertex_size = (self.current_vertex_offset as u64
                    * std::mem::size_of::<Vertex>() as u64)
                    + vertex_size;
                let required_index_size = (self.current_index_offset as u64
                    * std::mem::size_of::<u32>() as u64)
                    + index_size;

                if required_vertex_size > MAX_BUFFER_SIZE || required_index_size > MAX_BUFFER_SIZE {
                    tracing::warn!(
                        "Shadow mesh {} exceeds maximum buffer size, skipping shadow rendering",
                        name
                    );
                    continue;
                }

                if required_vertex_size > self.vertex_buffer_size {
                    let new_size = (required_vertex_size * 2)
                        .max(self.vertex_buffer_size * 2)
                        .min(MAX_BUFFER_SIZE);
                    let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Shadow Vertex Buffer (Resized)"),
                        size: new_size,
                        usage: wgpu::BufferUsages::VERTEX
                            | wgpu::BufferUsages::STORAGE
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::COPY_SRC,
                        mapped_at_creation: false,
                    });

                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Shadow Vertex Buffer Copy"),
                        });
                    encoder.copy_buffer_to_buffer(
                        &self.vertex_buffer,
                        0,
                        &new_buffer,
                        0,
                        self.vertex_buffer_size,
                    );
                    queue.submit(Some(encoder.finish()));

                    self.vertex_buffer = new_buffer;
                    self.vertex_buffer_size = new_size;
                    self.vertex_buffer_generation += 1;
                }

                if required_index_size > self.index_buffer_size {
                    let new_size = (required_index_size * 2)
                        .max(self.index_buffer_size * 2)
                        .min(MAX_BUFFER_SIZE);
                    let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("Shadow Index Buffer (Resized)"),
                        size: new_size,
                        usage: wgpu::BufferUsages::INDEX
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::COPY_SRC,
                        mapped_at_creation: false,
                    });

                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Shadow Index Buffer Copy"),
                        });
                    encoder.copy_buffer_to_buffer(
                        &self.index_buffer,
                        0,
                        &new_buffer,
                        0,
                        self.index_buffer_size,
                    );
                    queue.submit(Some(encoder.finish()));

                    self.index_buffer = new_buffer;
                    self.index_buffer_size = new_size;
                }

                queue.write_buffer(
                    &self.vertex_buffer,
                    self.current_vertex_offset as u64 * std::mem::size_of::<Vertex>() as u64,
                    bytemuck::cast_slice(&mesh.vertices),
                );

                queue.write_buffer(
                    &self.index_buffer,
                    self.current_index_offset as u64 * std::mem::size_of::<u32>() as u64,
                    bytemuck::cast_slice(&mesh.indices),
                );

                self.meshes.insert(
                    name.clone(),
                    (
                        self.current_vertex_offset,
                        mesh.vertices.len() as u32,
                        self.current_index_offset,
                        mesh.indices.len() as u32,
                    ),
                );

                self.culling.register_mesh(
                    name.as_str(),
                    mesh.indices.len() as u32,
                    self.current_index_offset,
                    self.current_vertex_offset as i32,
                    &mesh.vertices,
                );

                self.current_vertex_offset += mesh.vertices.len() as u32;
                self.current_index_offset += mesh.indices.len() as u32;
            }
        }

        if compacted {
            self.shrink_shadow_buffers(device, queue);
        }
    }

    /// Drops despawned meshes from the packed shadow buffers once dead geometry
    /// dominates the occupied range. The static shadow vertex and index buffers
    /// are bump allocated keyed by mesh name, so a despawned piece leaves a hole
    /// that the add path never reuses. Without reclamation a churning workload
    /// climbs the offset until it crosses `MAX_BUFFER_SIZE` and shadows start
    /// getting skipped. Clearing the name map, the offsets, and the culling geo
    /// table lets the add loop rebuild a dense layout from the live cache, after
    /// which `shrink_shadow_buffers` trims the now oversized allocations.
    ///
    /// The full check runs on a periodic timer and, additionally, on every frame
    /// once the buffer is over half full so a despawn burst that pushes toward
    /// the cap is reclaimed immediately rather than after the timer elapses.
    /// Returns whether a rebuild was started so the caller shrinks afterward.
    fn compact_shadow_meshes_if_warranted(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_cache: &crate::mesh_cache::MeshCache,
    ) -> bool {
        let used_vertex_bytes =
            self.current_vertex_offset as u64 * std::mem::size_of::<Vertex>() as u64;
        let used_index_bytes = self.current_index_offset as u64 * std::mem::size_of::<u32>() as u64;

        self.compaction_counter += 1;
        let timer_elapsed = self.compaction_counter >= COMPACTION_CHECK_INTERVAL_FRAMES;
        let near_cap =
            used_vertex_bytes * 2 > MAX_BUFFER_SIZE || used_index_bytes * 2 > MAX_BUFFER_SIZE;
        if !timer_elapsed && !near_cap {
            return false;
        }
        if timer_elapsed {
            self.compaction_counter = 0;
        }

        let mut live_vertex_bytes = 0u64;
        let mut live_index_bytes = 0u64;
        for (_, mesh) in mesh_cache_iter(mesh_cache) {
            if mesh.skin_data.is_some() {
                continue;
            }
            live_vertex_bytes += (mesh.vertices.len() * std::mem::size_of::<Vertex>()) as u64;
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

        self.meshes.clear();
        self.current_vertex_offset = 0;
        self.current_index_offset = 0;
        self.culling.clear_mesh_table();
        self.add_builtin_meshes(device, queue);
        true
    }

    /// Reallocates the static shadow vertex and index buffers down toward the
    /// range the rebuilt mesh set actually occupies, copying the live prefix
    /// across and leaving one growth step of headroom so the next add does not
    /// immediately reallocate.
    fn shrink_shadow_buffers(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let used_vertex_bytes =
            self.current_vertex_offset as u64 * std::mem::size_of::<Vertex>() as u64;
        let used_index_bytes = self.current_index_offset as u64 * std::mem::size_of::<u32>() as u64;

        let new_vertex_size = (used_vertex_bytes * 2).min(MAX_BUFFER_SIZE);
        if used_vertex_bytes > 0 && new_vertex_size < self.vertex_buffer_size {
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Vertex Buffer (Compacted)"),
                size: new_vertex_size,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Vertex Buffer Compaction"),
            });
            encoder.copy_buffer_to_buffer(
                &self.vertex_buffer,
                0,
                &new_buffer,
                0,
                used_vertex_bytes,
            );
            queue.submit(Some(encoder.finish()));
            self.vertex_buffer = new_buffer;
            self.vertex_buffer_size = new_vertex_size;
        }

        let new_index_size = (used_index_bytes * 2).min(MAX_BUFFER_SIZE);
        if used_index_bytes > 0 && new_index_size < self.index_buffer_size {
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Index Buffer (Compacted)"),
                size: new_index_size,
                usage: wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Index Buffer Compaction"),
            });
            encoder.copy_buffer_to_buffer(&self.index_buffer, 0, &new_buffer, 0, used_index_bytes);
            queue.submit(Some(encoder.finish()));
            self.index_buffer = new_buffer;
            self.index_buffer_size = new_index_size;
        }
    }

    pub fn sync_skinned_meshes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_cache: &crate::mesh_cache::MeshCache,
    ) {
        let compacted = self.compact_skinned_shadow_meshes_if_warranted(mesh_cache);

        for (name, mesh) in mesh_cache_iter(mesh_cache) {
            if mesh.skin_data.is_none() {
                continue;
            }

            if self.skinned_meshes.contains_key(name.as_str()) {
                continue;
            }

            let skin_data = mesh.skin_data.as_ref().unwrap();
            let skinned_vertices = &skin_data.skinned_vertices;

            let vertex_size =
                (skinned_vertices.len() * std::mem::size_of::<SkinnedVertex>()) as u64;
            let index_size = (mesh.indices.len() * std::mem::size_of::<u32>()) as u64;

            let required_vertex_size = (self.current_skinned_vertex_offset as u64
                * std::mem::size_of::<SkinnedVertex>() as u64)
                + vertex_size;
            let required_index_size = (self.current_skinned_index_offset as u64
                * std::mem::size_of::<u32>() as u64)
                + index_size;

            if required_vertex_size > MAX_BUFFER_SIZE || required_index_size > MAX_BUFFER_SIZE {
                tracing::warn!(
                    "Shadow skinned mesh {} exceeds maximum buffer size, skipping shadow rendering",
                    name
                );
                continue;
            }

            if required_vertex_size > self.skinned_vertex_buffer_size {
                let new_size = (required_vertex_size * 2)
                    .max(self.skinned_vertex_buffer_size * 2)
                    .min(MAX_BUFFER_SIZE);
                let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Shadow Skinned Vertex Buffer (Resized)"),
                    size: new_size,
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Shadow Skinned Vertex Buffer Copy"),
                });
                encoder.copy_buffer_to_buffer(
                    &self.skinned_vertex_buffer,
                    0,
                    &new_buffer,
                    0,
                    self.skinned_vertex_buffer_size,
                );
                queue.submit(Some(encoder.finish()));

                self.skinned_vertex_buffer = new_buffer;
                self.skinned_vertex_buffer_size = new_size;
            }

            if required_index_size > self.skinned_index_buffer_size {
                let new_size = (required_index_size * 2)
                    .max(self.skinned_index_buffer_size * 2)
                    .min(MAX_BUFFER_SIZE);
                let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Shadow Skinned Index Buffer (Resized)"),
                    size: new_size,
                    usage: wgpu::BufferUsages::INDEX
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Shadow Skinned Index Buffer Copy"),
                });
                encoder.copy_buffer_to_buffer(
                    &self.skinned_index_buffer,
                    0,
                    &new_buffer,
                    0,
                    self.skinned_index_buffer_size,
                );
                queue.submit(Some(encoder.finish()));

                self.skinned_index_buffer = new_buffer;
                self.skinned_index_buffer_size = new_size;
            }

            queue.write_buffer(
                &self.skinned_vertex_buffer,
                self.current_skinned_vertex_offset as u64
                    * std::mem::size_of::<SkinnedVertex>() as u64,
                bytemuck::cast_slice(skinned_vertices),
            );

            queue.write_buffer(
                &self.skinned_index_buffer,
                self.current_skinned_index_offset as u64 * std::mem::size_of::<u32>() as u64,
                bytemuck::cast_slice(&mesh.indices),
            );

            self.skinned_meshes.insert(
                name.clone(),
                (
                    self.current_skinned_vertex_offset,
                    skinned_vertices.len() as u32,
                    self.current_skinned_index_offset,
                    mesh.indices.len() as u32,
                ),
            );

            self.current_skinned_vertex_offset += skinned_vertices.len() as u32;
            self.current_skinned_index_offset += mesh.indices.len() as u32;
        }

        if compacted {
            self.shrink_skinned_shadow_buffers(device, queue);
        }
    }

    /// Reclaims the skinned shadow buffers once despawned skinned geometry
    /// dominates the occupied range, mirroring the static path. Skinned meshes
    /// carry no built-in primitives and no persistent culling table, so the
    /// rebuild simply clears the name map and offsets and lets the add loop
    /// repack the live cache. The per-frame skinned object buffer is rebuilt
    /// from this map every frame, so the reassigned offsets take effect at once.
    ///
    /// Like the static path, the check runs on a periodic timer and on every
    /// frame once the buffer is over half full so a despawn burst is reclaimed
    /// immediately rather than after the timer elapses.
    /// Returns whether a rebuild was started so the caller shrinks afterward.
    fn compact_skinned_shadow_meshes_if_warranted(
        &mut self,
        mesh_cache: &crate::mesh_cache::MeshCache,
    ) -> bool {
        let used_vertex_bytes =
            self.current_skinned_vertex_offset as u64 * std::mem::size_of::<SkinnedVertex>() as u64;
        let used_index_bytes =
            self.current_skinned_index_offset as u64 * std::mem::size_of::<u32>() as u64;

        self.skinned_compaction_counter += 1;
        let timer_elapsed = self.skinned_compaction_counter >= COMPACTION_CHECK_INTERVAL_FRAMES;
        let near_cap =
            used_vertex_bytes * 2 > MAX_BUFFER_SIZE || used_index_bytes * 2 > MAX_BUFFER_SIZE;
        if !timer_elapsed && !near_cap {
            return false;
        }
        if timer_elapsed {
            self.skinned_compaction_counter = 0;
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

        self.skinned_meshes.clear();
        self.current_skinned_vertex_offset = 0;
        self.current_skinned_index_offset = 0;
        true
    }

    /// Reallocates the skinned shadow vertex and index buffers down toward the
    /// range the rebuilt mesh set occupies, copying the live prefix across and
    /// leaving one growth step of headroom.
    fn shrink_skinned_shadow_buffers(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let used_vertex_bytes =
            self.current_skinned_vertex_offset as u64 * std::mem::size_of::<SkinnedVertex>() as u64;
        let used_index_bytes =
            self.current_skinned_index_offset as u64 * std::mem::size_of::<u32>() as u64;

        let new_vertex_size = (used_vertex_bytes * 2).min(MAX_BUFFER_SIZE);
        if used_vertex_bytes > 0 && new_vertex_size < self.skinned_vertex_buffer_size {
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Skinned Vertex Buffer (Compacted)"),
                size: new_vertex_size,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Skinned Vertex Buffer Compaction"),
            });
            encoder.copy_buffer_to_buffer(
                &self.skinned_vertex_buffer,
                0,
                &new_buffer,
                0,
                used_vertex_bytes,
            );
            queue.submit(Some(encoder.finish()));
            self.skinned_vertex_buffer = new_buffer;
            self.skinned_vertex_buffer_size = new_vertex_size;
        }

        let new_index_size = (used_index_bytes * 2).min(MAX_BUFFER_SIZE);
        if used_index_bytes > 0 && new_index_size < self.skinned_index_buffer_size {
            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Skinned Index Buffer (Compacted)"),
                size: new_index_size,
                usage: wgpu::BufferUsages::INDEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shadow Skinned Index Buffer Compaction"),
            });
            encoder.copy_buffer_to_buffer(
                &self.skinned_index_buffer,
                0,
                &new_buffer,
                0,
                used_index_bytes,
            );
            queue.submit(Some(encoder.finish()));
            self.skinned_index_buffer = new_buffer;
            self.skinned_index_buffer_size = new_index_size;
        }
    }

    pub fn rebuild_skinned_bind_group(&mut self, device: &wgpu::Device) {
        self.skinned_bind_group = buffer_bind_group(
            device,
            "Shadow Skinned Bind Group",
            &self.skinned_bind_group_layout,
            &[
                &self.skinned_object_buffer,
                &self.joint_matrices_buffer,
                &self.skinned_visible_indices_buffer,
            ],
        );
    }

    pub fn rebuild_skinned_cull_bind_groups(&mut self, device: &wgpu::Device) {
        self.skinned_cull_bind_groups = self
            .skinned_cull_uniform_buffers
            .iter()
            .map(|uniform_buffer| {
                buffer_bind_group(
                    device,
                    "Shadow Skinned Cull Bind Group",
                    &self.skinned_cull_bind_group_layout,
                    &[
                        &self.skinned_cull_objects_buffer,
                        &self.skinned_indirect_buffer,
                        &self.skinned_visible_indices_buffer,
                        uniform_buffer,
                    ],
                )
            })
            .collect();
    }

    pub fn rebuild_skinning_compute_bind_group(&mut self, device: &wgpu::Device) {
        self.skinning_compute_bind_group = buffer_bind_group(
            device,
            "Shadow Skinning Compute Bind Group",
            &self.skinning_compute_bind_group_layout,
            &[
                &self.bone_transforms_buffer,
                &self.inverse_bind_matrices_buffer,
                &self.skin_data_buffer,
                &self.joint_matrices_buffer,
            ],
        );
    }

    pub fn spotlight_shadow_slots(&self) -> &[SpotlightShadowSlot] {
        &self.spotlight_shadow_slots
    }

    pub fn spotlight_shadow_data(&self) -> &[SpotlightShadowData] {
        &self.spotlight_shadow_data
    }

    pub fn cascade_data(&self) -> &CascadeData {
        &self.cascade_data
    }

    pub fn point_light_shadow_slots(&self) -> &[PointLightShadowSlot] {
        &self.point_light_shadow_slots
    }

    pub fn point_light_shadow_data(&self) -> &[PointLightShadowData] {
        &self.point_light_shadow_data
    }

    pub fn point_light_cubemap_view(&self) -> &wgpu::TextureView {
        &self.point_light_cubemap_view
    }
}
