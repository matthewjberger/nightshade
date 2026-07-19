use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};
use wgpu::{BindGroup, BindGroupLayout, ComputePipeline, RenderPipeline};

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const INITIAL_LINE_CAPACITY: u32 = 1024;
const INITIAL_BV_CAPACITY: u32 = 1024;
const INITIAL_NORMAL_CAPACITY: u32 = 1024;
const BUFFER_GROWTH_FACTOR: f32 = 2.0;
/// Upper bound on debug lines drawn in one frame.
pub const MAX_LINES: u32 = 1_000_000;
/// Upper bound on bounding-volume boxes per frame.
pub const MAX_BOUNDING_VOLUMES: u32 = 100_000;
/// Upper bound on normal-visualization lines per frame.
pub const MAX_NORMALS: u32 = 1_000_000;

/// GPU indexed indirect draw command matching the wgpu layout.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndexedIndirectCommand {
    /// Indices drawn per instance.
    pub index_count: u32,
    /// Instances drawn.
    pub instance_count: u32,
    /// First index read from the index buffer.
    pub first_index: u32,
    /// Value added to each index before fetching a vertex.
    pub vertex_offset: i32,
    /// First instance id.
    pub first_instance: u32,
}

/// One oriented bounding volume expanded into wireframe lines on the GPU.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuBoundingVolumeData {
    /// Box center in local space.
    pub center: [f32; 3],
    /// Padding for 16-byte alignment.
    pub _pad0: f32,
    /// Half-size along each local axis.
    pub half_extents: [f32; 3],
    /// Padding for 16-byte alignment.
    pub _pad1: f32,
    /// Orientation quaternion as (x, y, z, w).
    pub orientation: [f32; 4],
    /// World transform applied to the box.
    pub transform: [[f32; 4]; 4],
    /// Line color as linear RGBA.
    pub color: [f32; 4],
}

/// Uniform inputs for the bounding-volume line generation compute pass.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BoundingVolumeParams {
    /// Number of bounding volumes to expand.
    pub bounding_volume_count: u32,
    /// First line-buffer slot the generated lines are written to.
    pub line_offset: u32,
    /// Padding for 16-byte alignment.
    pub _padding: [u32; 2],
}

/// One vertex normal expanded into a line segment on the GPU.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuNormalData {
    /// Normal origin in local space.
    pub position: [f32; 3],
    /// Padding for 16-byte alignment.
    pub _pad0: f32,
    /// Normal direction in local space.
    pub normal: [f32; 3],
    /// Padding for 16-byte alignment.
    pub _pad1: f32,
    /// World transform applied to the origin and direction.
    pub transform: [[f32; 4]; 4],
    /// Line color as linear RGBA.
    pub color: [f32; 4],
    /// Length of the drawn normal line.
    pub length: f32,
    /// Padding for 16-byte alignment.
    pub _pad2: [f32; 3],
}

/// Uniform inputs for the normal line generation compute pass.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NormalParams {
    pub(crate) normal_count: u32,
    /// First line-buffer slot the generated lines are written to.
    pub line_offset: u32,
    /// Padding for 16-byte alignment.
    pub _padding: [u32; 2],
}

/// One debug line segment stored in the shared line buffer.
#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuLineData {
    /// Segment start position, w unused.
    pub start: [f32; 4],
    /// Segment end position, w unused.
    pub end: [f32; 4],
    /// Line color as linear RGBA.
    pub color: [f32; 4],
    /// Owning entity id for picking.
    pub entity_id: u32,
    /// Nonzero when the line is drawn.
    pub visible: u32,
    /// Depth test mode selector.
    pub depth_mode: u32,
    /// Padding for 16-byte alignment.
    pub _padding: u32,
}

/// Uniform inputs for the line frustum-culling compute pass.
#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CullingData {
    /// Combined view-projection matrix.
    pub view_proj: [[f32; 4]; 4],
    /// Six frustum planes as (a, b, c, d).
    pub frustum_planes: [[f32; 4]; 6],
    pub(crate) line_count: u32,
    /// Padding for 16-byte alignment.
    pub _padding: [u32; 15],
}

/// Vertex of the unit primitive instancing stretches into each line segment.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineVertex {
    /// Endpoint position, x is 0 at the segment start and 1 at the end.
    pub position: [f32; 3],
}

/// View-projection uniform for the line render pipeline.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineUniform {
    /// Combined view-projection matrix.
    pub view_proj: [[f32; 4]; 4],
}

impl From<&crate::wgpu::render_configs::RenderLine> for GpuLineData {
    fn from(line: &crate::wgpu::render_configs::RenderLine) -> Self {
        Self {
            start: [line.start.x, line.start.y, line.start.z, 1.0],
            end: [line.end.x, line.end.y, line.end.z, 1.0],
            color: [line.color.x, line.color.y, line.color.z, line.color.w],
            entity_id: line.entity_id,
            visible: 1,
            depth_mode: line.depth_mode,
            _padding: 0,
        }
    }
}

impl From<&crate::wgpu::render_configs::RenderBoundingVolume> for GpuBoundingVolumeData {
    fn from(volume: &crate::wgpu::render_configs::RenderBoundingVolume) -> Self {
        Self {
            center: [volume.center.x, volume.center.y, volume.center.z],
            _pad0: 0.0,
            half_extents: [
                volume.half_extents.x,
                volume.half_extents.y,
                volume.half_extents.z,
            ],
            _pad1: 0.0,
            orientation: [
                volume.orientation.x,
                volume.orientation.y,
                volume.orientation.z,
                volume.orientation.w,
            ],
            transform: volume.transform.into(),
            color: [
                volume.color.x,
                volume.color.y,
                volume.color.z,
                volume.color.w,
            ],
        }
    }
}

impl From<&crate::wgpu::render_configs::RenderNormal> for GpuNormalData {
    fn from(normal: &crate::wgpu::render_configs::RenderNormal) -> Self {
        Self {
            position: [normal.position.x, normal.position.y, normal.position.z],
            _pad0: 0.0,
            normal: [normal.normal.x, normal.normal.y, normal.normal.z],
            _pad1: 0.0,
            transform: normal.transform.into(),
            color: [
                normal.color.x,
                normal.color.y,
                normal.color.z,
                normal.color.w,
            ],
            length: normal.length,
            _pad2: [0.0, 0.0, 0.0],
        }
    }
}

/// GPU-driven debug line pass drawing lines, bounding volumes, and normals.
pub struct LinesPass {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    line_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    storage_bind_group_layout: BindGroupLayout,
    storage_bind_group: BindGroup,
    pipeline: RenderPipeline,
    draw_commands_buffer: wgpu::Buffer,
    draw_count_buffer: wgpu::Buffer,
    culling_bind_group_layout: BindGroupLayout,
    culling_bind_group: BindGroup,
    culling_pipeline: ComputePipeline,
    culling_data_buffer: wgpu::Buffer,
    pub(crate) line_count: u32,
    pub(crate) current_capacity: u32,
    bv_buffer: wgpu::Buffer,
    bv_params_buffer: wgpu::Buffer,
    bv_bind_group_layout: BindGroupLayout,
    bv_bind_group: BindGroup,
    bv_pipeline: ComputePipeline,
    pub(crate) bv_count: u32,
    pub(crate) bv_capacity: u32,
    pub(crate) bv_line_count: u32,
    normal_buffer: wgpu::Buffer,
    normal_params_buffer: wgpu::Buffer,
    normal_bind_group_layout: BindGroupLayout,
    normal_bind_group: BindGroup,
    normal_pipeline: ComputePipeline,
    pub(crate) normal_count: u32,
    pub(crate) normal_capacity: u32,
    pub(crate) normal_line_count: u32,
}

/// Builds a `main`-entry compute pipeline from a single bind group layout.
fn main_compute_pipeline(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
) -> wgpu::ComputePipeline {
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    })
}

impl LinesPass {
    /// Creates the pass, its pipelines, and the initial GPU buffers.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let vertices = [
            LineVertex {
                position: [0.0, 0.0, 0.0],
            },
            LineVertex {
                position: [1.0, 0.0, 0.0],
            },
        ];

        let vertex_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Line Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );

        let indices = [0u32, 1u32];
        let index_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Line Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        );

        let line_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Buffer"),
            size: (std::mem::size_of::<GpuLineData>() * INITIAL_LINE_CAPACITY as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Uniform Buffer"),
            size: std::mem::size_of::<LineUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("Line Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Line Bind Group"),
        });

        let storage_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("Line Storage Bind Group Layout"),
            });

        let storage_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &storage_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: line_buffer.as_entire_binding(),
            }],
            label: Some("Line Storage Bind Group"),
        });

        let shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "line_gpu.wgsl",
            include_str!("../../shaders/line_gpu.wgsl"),
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Line Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout), Some(&storage_bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let draw_commands_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Draw Commands Buffer"),
            size: (std::mem::size_of::<DrawIndexedIndirectCommand>() * MAX_LINES as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let draw_count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Draw Count Buffer"),
            size: std::mem::size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let culling_data_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Culling Data Buffer"),
            size: std::mem::size_of::<CullingData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let culling_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "Line Culling Shader",
            include_str!("../../shaders/line_culling_gpu.wgsl"),
        );

        let culling_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Line Culling Bind Group Layout"),
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
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
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

        let culling_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Line Culling Bind Group"),
            layout: &culling_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: line_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: draw_commands_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: draw_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: culling_data_buffer.as_entire_binding(),
                },
            ],
        });

        let culling_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Line Culling Pipeline Layout"),
                bind_group_layouts: &[Some(&culling_bind_group_layout)],
                immediate_size: 0,
            });

        let culling_pipeline = main_compute_pipeline(
            device,
            "Line Culling Pipeline",
            &culling_pipeline_layout,
            &culling_shader,
        );

        let bv_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bounding Volume Buffer"),
            size: (std::mem::size_of::<GpuBoundingVolumeData>() * INITIAL_BV_CAPACITY as usize)
                as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bv_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bounding Volume Params Buffer"),
            size: std::mem::size_of::<BoundingVolumeParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bv_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bounding Volume Bind Group Layout"),
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
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
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

        let bv_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bounding Volume Bind Group"),
            layout: &bv_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: bv_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: line_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: bv_params_buffer.as_entire_binding(),
                },
            ],
        });

        let bv_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "Bounding Volume Lines Shader",
            include_str!("../../shaders/bounding_volume_lines.wgsl"),
        );

        let bv_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Bounding Volume Pipeline Layout"),
            bind_group_layouts: &[Some(&bv_bind_group_layout)],
            immediate_size: 0,
        });

        let bv_pipeline = main_compute_pipeline(
            device,
            "Bounding Volume Lines Pipeline",
            &bv_pipeline_layout,
            &bv_shader,
        );

        let normal_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Normal Buffer"),
            size: (std::mem::size_of::<GpuNormalData>() * INITIAL_NORMAL_CAPACITY as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let normal_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Normal Params Buffer"),
            size: std::mem::size_of::<NormalParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let normal_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Normal Bind Group Layout"),
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
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
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

        let normal_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Normal Bind Group"),
            layout: &normal_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: normal_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: line_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: normal_params_buffer.as_entire_binding(),
                },
            ],
        });

        let normal_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "Normal Lines Shader",
            include_str!("../../shaders/normal_lines.wgsl"),
        );

        let normal_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Normal Pipeline Layout"),
                bind_group_layouts: &[Some(&normal_bind_group_layout)],
                immediate_size: 0,
            });

        let normal_pipeline = main_compute_pipeline(
            device,
            "Normal Lines Pipeline",
            &normal_pipeline_layout,
            &normal_shader,
        );

        Self {
            vertex_buffer,
            index_buffer,
            line_buffer,
            uniform_buffer,
            bind_group,
            storage_bind_group_layout,
            storage_bind_group,
            pipeline,
            draw_commands_buffer,
            draw_count_buffer,
            culling_bind_group_layout,
            culling_bind_group,
            culling_pipeline,
            culling_data_buffer,
            line_count: 0,
            current_capacity: INITIAL_LINE_CAPACITY,
            bv_buffer,
            bv_params_buffer,
            bv_bind_group_layout,
            bv_bind_group,
            bv_pipeline,
            bv_count: 0,
            bv_capacity: INITIAL_BV_CAPACITY,
            bv_line_count: 0,
            normal_buffer,
            normal_params_buffer,
            normal_bind_group_layout,
            normal_bind_group,
            normal_pipeline,
            normal_count: 0,
            normal_capacity: INITIAL_NORMAL_CAPACITY,
            normal_line_count: 0,
        }
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, required_lines: u32) {
        if required_lines <= self.current_capacity {
            return;
        }

        let new_capacity = (required_lines as f32 * BUFFER_GROWTH_FACTOR).ceil() as u32;
        let new_capacity = new_capacity.min(MAX_LINES);

        tracing::info!(
            "Growing Line buffers from {} to {} lines ({} -> {} bytes)",
            self.current_capacity,
            new_capacity,
            self.current_capacity as u64 * std::mem::size_of::<GpuLineData>() as u64,
            new_capacity as u64 * std::mem::size_of::<GpuLineData>() as u64
        );

        let new_line_buffer_size = std::mem::size_of::<GpuLineData>() * new_capacity as usize;
        let new_line_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Buffer (Resized)"),
            size: new_line_buffer_size as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let old_line_data_size = std::mem::size_of::<GpuLineData>() * self.line_count as usize;
        if old_line_data_size > 0 {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Line Buffer Resize Copy"),
            });

            encoder.copy_buffer_to_buffer(
                &self.line_buffer,
                0,
                &new_line_buffer,
                0,
                old_line_data_size as u64,
            );

            queue.submit(std::iter::once(encoder.finish()));
        }

        self.line_buffer = new_line_buffer;

        self.storage_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.storage_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.line_buffer.as_entire_binding(),
            }],
            label: Some("Line Storage Bind Group"),
        });

        self.culling_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Line Culling Bind Group"),
            layout: &self.culling_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.line_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.draw_commands_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.draw_count_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.culling_data_buffer.as_entire_binding(),
                },
            ],
        });

        self.bv_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bounding Volume Bind Group"),
            layout: &self.bv_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.bv_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.line_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.bv_params_buffer.as_entire_binding(),
                },
            ],
        });

        self.normal_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Normal Bind Group"),
            layout: &self.normal_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.normal_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.line_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.normal_params_buffer.as_entire_binding(),
                },
            ],
        });

        self.current_capacity = new_capacity;
    }

    fn ensure_bv_capacity(&mut self, device: &wgpu::Device, required_bvs: u32) {
        if required_bvs <= self.bv_capacity {
            return;
        }

        let new_capacity = (required_bvs as f32 * BUFFER_GROWTH_FACTOR).ceil() as u32;
        let new_capacity = new_capacity.min(MAX_BOUNDING_VOLUMES);

        self.bv_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bounding Volume Buffer (Resized)"),
            size: (std::mem::size_of::<GpuBoundingVolumeData>() * new_capacity as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.bv_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bounding Volume Bind Group"),
            layout: &self.bv_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.bv_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.line_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.bv_params_buffer.as_entire_binding(),
                },
            ],
        });

        self.bv_capacity = new_capacity;
    }

    fn update_bounding_volumes(&mut self, queue: &wgpu::Queue, bv_data: &[GpuBoundingVolumeData]) {
        self.bv_count = bv_data.len() as u32;
        self.bv_line_count = self.bv_count * 12;

        if !bv_data.is_empty() {
            queue.write_buffer(&self.bv_buffer, 0, bytemuck::cast_slice(bv_data));

            let params = BoundingVolumeParams {
                bounding_volume_count: self.bv_count,
                line_offset: self.line_count,
                _padding: [0, 0],
            };
            queue.write_buffer(&self.bv_params_buffer, 0, bytemuck::cast_slice(&[params]));
        }
    }

    /// Dispatches the compute pass expanding bounding volumes into lines.
    pub fn generate_bv_lines(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.bv_count == 0 {
            return;
        }

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Bounding Volume Lines Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.bv_pipeline);
        compute_pass.set_bind_group(0, &self.bv_bind_group, &[]);

        let workgroup_size = 64;
        let dispatch_size = self.bv_count.div_ceil(workgroup_size);
        compute_pass.dispatch_workgroups(dispatch_size, 1, 1);
    }

    fn ensure_normal_capacity(&mut self, device: &wgpu::Device, required_normals: u32) {
        if required_normals <= self.normal_capacity {
            return;
        }

        let new_capacity = (required_normals as f32 * BUFFER_GROWTH_FACTOR).ceil() as u32;
        let new_capacity = new_capacity.min(MAX_NORMALS);

        self.normal_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Normal Buffer (Resized)"),
            size: (std::mem::size_of::<GpuNormalData>() * new_capacity as usize) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.normal_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Normal Bind Group"),
            layout: &self.normal_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.normal_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.line_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.normal_params_buffer.as_entire_binding(),
                },
            ],
        });

        self.normal_capacity = new_capacity;
    }

    fn update_normals(&mut self, queue: &wgpu::Queue, normal_data: &[GpuNormalData]) {
        self.normal_count = normal_data.len() as u32;
        self.normal_line_count = self.normal_count;

        if !normal_data.is_empty() {
            queue.write_buffer(&self.normal_buffer, 0, bytemuck::cast_slice(normal_data));

            let params = NormalParams {
                normal_count: self.normal_count,
                line_offset: self.line_count + self.bv_line_count,
                _padding: [0, 0],
            };
            queue.write_buffer(
                &self.normal_params_buffer,
                0,
                bytemuck::cast_slice(&[params]),
            );
        }
    }

    /// Dispatches the compute pass expanding normals into lines.
    pub fn generate_normal_lines(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.normal_count == 0 {
            return;
        }

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Normal Lines Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.normal_pipeline);
        compute_pass.set_bind_group(0, &self.normal_bind_group, &[]);

        let workgroup_size = 256;
        let dispatch_size = self.normal_count.div_ceil(workgroup_size);
        compute_pass.dispatch_workgroups(dispatch_size, 1, 1);
    }

    /// Total lines to draw across debug lines, bounding volumes, and normals.
    pub fn total_line_count(&self) -> u32 {
        self.line_count + self.bv_line_count + self.normal_line_count
    }

    /// Writes the view-projection matrix to the render uniform buffer.
    pub fn update_uniforms(&self, queue: &wgpu::Queue, view_proj: nalgebra_glm::Mat4) {
        let uniform = LineUniform {
            view_proj: view_proj.into(),
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Writes the view-projection and frustum planes to the culling buffer.
    pub fn update_culling_data(
        &self,
        queue: &wgpu::Queue,
        view_proj: nalgebra_glm::Mat4,
        frustum_planes: [nalgebra_glm::Vec4; 6],
    ) {
        let culling_data = CullingData {
            view_proj: view_proj.into(),
            frustum_planes: frustum_planes.map(|v| [v.x, v.y, v.z, v.w]),
            line_count: self.total_line_count(),
            _padding: [0; 15],
        };
        queue.write_buffer(
            &self.culling_data_buffer,
            0,
            bytemuck::cast_slice(&[culling_data]),
        );
    }

    /// Uploads the frame's debug lines, growing the shared line buffer to
    /// hold them alongside the bounding-volume region.
    pub fn upload_lines(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        lines: &[crate::wgpu::render_configs::RenderLine],
    ) {
        let line_data: Vec<GpuLineData> = lines.iter().map(GpuLineData::from).collect();
        let line_count = line_data.len() as u32;
        let total_needed = line_count + self.bv_line_count;
        if total_needed > self.current_capacity {
            self.ensure_capacity(device, queue, total_needed);
        }
        self.update_line_data(queue, &line_data);
    }

    /// Uploads the frame's bounding-volume boxes, growing the box and line
    /// buffers as needed.
    pub fn upload_bounding_volumes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        volumes: &[crate::wgpu::render_configs::RenderBoundingVolume],
    ) {
        let bv_data: Vec<GpuBoundingVolumeData> =
            volumes.iter().map(GpuBoundingVolumeData::from).collect();
        let bv_count = bv_data.len() as u32;
        if bv_count > self.bv_capacity {
            self.ensure_bv_capacity(device, bv_count);
        }
        let total_lines_needed = self.line_count + bv_count * 12;
        if total_lines_needed > self.current_capacity {
            self.ensure_capacity(device, queue, total_lines_needed);
        }
        self.update_bounding_volumes(queue, &bv_data);
    }

    /// Uploads the frame's normal-visualization lines, growing the normal and
    /// line buffers as needed.
    pub fn upload_normals(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        normals: &[crate::wgpu::render_configs::RenderNormal],
    ) {
        let normal_data: Vec<GpuNormalData> = normals.iter().map(GpuNormalData::from).collect();
        let normal_count = normal_data.len() as u32;
        if normal_count > self.normal_capacity {
            self.ensure_normal_capacity(device, normal_count);
        }
        let total_lines_needed = self.line_count + self.bv_line_count + normal_count;
        if total_lines_needed > self.current_capacity {
            self.ensure_capacity(device, queue, total_lines_needed);
        }
        self.update_normals(queue, &normal_data);
    }

    /// Drops the current frame's bounding-volume lines.
    pub fn clear_bounding_volumes(&mut self) {
        self.bv_count = 0;
        self.bv_line_count = 0;
    }

    /// Drops the current frame's normal lines.
    pub fn clear_normals(&mut self) {
        self.normal_count = 0;
        self.normal_line_count = 0;
    }

    fn update_line_data(&mut self, queue: &wgpu::Queue, line_data: &[GpuLineData]) {
        if !line_data.is_empty() {
            let clamped_count = (line_data.len() as u32).min(self.current_capacity);
            self.line_count = clamped_count;
            queue.write_buffer(
                &self.line_buffer,
                0,
                bytemuck::cast_slice(&line_data[..clamped_count as usize]),
            );
        } else {
            self.line_count = 0;
        }
    }

    /// Encodes the line-generation and, when enabled, culling compute passes.
    pub fn encode_compute(&self, encoder: &mut wgpu::CommandEncoder, culling_enabled: bool) {
        if self.total_line_count() == 0 {
            return;
        }

        self.generate_bv_lines(encoder);
        self.generate_normal_lines(encoder);

        if culling_enabled {
            self.cull_lines(encoder);
        }
    }

    /// Records the line draw, using indirect culling output when enabled.
    pub fn record(&self, render_pass: &mut wgpu::RenderPass<'_>, culling_enabled: bool) {
        let total_lines = self.total_line_count();
        if total_lines == 0 {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, &self.storage_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        if culling_enabled {
            if cfg!(target_os = "macos") || cfg!(target_arch = "wasm32") {
                render_pass.multi_draw_indexed_indirect(&self.draw_commands_buffer, 0, total_lines);
            } else {
                render_pass.multi_draw_indexed_indirect_count(
                    &self.draw_commands_buffer,
                    0,
                    &self.draw_count_buffer,
                    0,
                    MAX_LINES,
                );
            }
        } else {
            render_pass.draw_indexed(0..2, 0, 0..total_lines);
        }
    }

    /// Dispatches the frustum-culling compute pass building the indirect draw commands.
    pub fn cull_lines(&self, encoder: &mut wgpu::CommandEncoder) {
        let total = self.total_line_count();
        if total > 0 {
            encoder.clear_buffer(&self.draw_count_buffer, 0, None);

            let commands_size =
                (total as u64) * std::mem::size_of::<DrawIndexedIndirectCommand>() as u64;
            encoder.clear_buffer(&self.draw_commands_buffer, 0, Some(commands_size));

            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Line Culling Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.culling_pipeline);
            compute_pass.set_bind_group(0, &self.culling_bind_group, &[]);

            let workgroup_size = 256;
            let dispatch_size = total.div_ceil(workgroup_size);
            let max_dispatch = 65535;
            let final_dispatch = dispatch_size.min(max_dispatch);

            compute_pass.dispatch_workgroups(final_dispatch, 1, 1);
        }
    }
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for LinesPass {
    fn name(&self) -> &str {
        "lines_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec![]
    }

    fn writes(&self) -> Vec<&str> {
        vec![]
    }

    fn reads_writes(&self) -> Vec<&str> {
        vec!["color", "depth"]
    }

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        if let Some(camera_matrices) = configs.scene.render_view.as_ref() {
            let view_proj = camera_matrices.projection * camera_matrices.view;

            self.update_uniforms(queue, view_proj);

            let frustum_planes = extract_frustum_planes(&view_proj);
            self.update_culling_data(queue, view_proj, frustum_planes);
        }
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::wgpu::render_configs::RenderInputs>,
    ) -> crate::wgpu::rendergraph::Result<Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>>
    {
        if self.total_line_count() == 0 {
            return Ok(context.into_sub_graph_commands());
        }

        let culling_enabled = context.configs.scene.gpu_culling_enabled;
        self.encode_compute(context.encoder, culling_enabled);

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Lines Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: color_load,
                        store: color_store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: depth_load,
                        store: depth_store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        self.record(&mut render_pass, culling_enabled);

        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}

/// Extracts the six normalized frustum planes from a view-projection matrix.
pub fn extract_frustum_planes(view_proj: &nalgebra_glm::Mat4) -> [nalgebra_glm::Vec4; 6] {
    let mut planes = [nalgebra_glm::Vec4::zeros(); 6];

    let row0 = nalgebra_glm::Vec4::new(
        view_proj[(0, 0)],
        view_proj[(0, 1)],
        view_proj[(0, 2)],
        view_proj[(0, 3)],
    );
    let row1 = nalgebra_glm::Vec4::new(
        view_proj[(1, 0)],
        view_proj[(1, 1)],
        view_proj[(1, 2)],
        view_proj[(1, 3)],
    );
    let row2 = nalgebra_glm::Vec4::new(
        view_proj[(2, 0)],
        view_proj[(2, 1)],
        view_proj[(2, 2)],
        view_proj[(2, 3)],
    );
    let row3 = nalgebra_glm::Vec4::new(
        view_proj[(3, 0)],
        view_proj[(3, 1)],
        view_proj[(3, 2)],
        view_proj[(3, 3)],
    );

    planes[0] = row3 + row0;
    planes[1] = row3 - row0;
    planes[2] = row3 + row1;
    planes[3] = row3 - row1;
    planes[4] = row3 + row2;
    planes[5] = row3 - row2;

    for plane in &mut planes {
        let normal_length = (plane.x * plane.x + plane.y * plane.y + plane.z * plane.z).sqrt();
        if normal_length > 1e-6 {
            *plane /= normal_length;
        }
    }

    planes
}
