use wgpu::util::DeviceExt;

use super::{
    GlobalUniforms, INITIAL_CHARACTER_COLOR_CAPACITY, INITIAL_RECT_CAPACITY,
    INITIAL_TEXT_INDEX_CAPACITY, INITIAL_TEXT_INSTANCE_CAPACITY, INITIAL_TEXT_VERTEX_CAPACITY,
    PaintPass, PaintPassVertex, PaintRectInstance, TextInstanceData, UiTextVertex,
};

impl PaintPass {
    /// Builds the rect and text pipelines, buffers, and bind groups for the UI pass.
    pub fn new(device: &wgpu::Device, color_format: wgpu::TextureFormat) -> Self {
        let global_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("PaintPass Global Uniform Layout"),
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

        let global_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PaintPass Global Uniform Buffer"),
            size: std::mem::size_of::<GlobalUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let global_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PaintPass Global Uniform Bind Group"),
            layout: &global_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_uniform_buffer.as_entire_binding(),
            }],
        });

        let rect_instance_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("PaintPass Rect Instance Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
                ],
            });

        let rect_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PaintPass Rect Instance Buffer"),
            size: (std::mem::size_of::<PaintRectInstance>() * INITIAL_RECT_CAPACITY) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rect_draw_order_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PaintPass Rect Draw Order Buffer"),
            size: (std::mem::size_of::<u32>() * INITIAL_RECT_CAPACITY) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rect_instance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PaintPass Rect Instance Bind Group"),
            layout: &rect_instance_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: rect_instance_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: rect_draw_order_buffer.as_entire_binding(),
                },
            ],
        });

        let rect_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "PaintPass Rect Shader",
            include_str!("../../../shaders/paint_rect.wgsl"),
        );

        let rect_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PaintPass Rect Pipeline Layout"),
            bind_group_layouts: &[
                Some(&global_uniform_bind_group_layout),
                Some(&rect_instance_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let vertices = [
            PaintPassVertex {
                position: [0.0, 0.0],
            },
            PaintPassVertex {
                position: [1.0, 0.0],
            },
            PaintPassVertex {
                position: [1.0, 1.0],
            },
            PaintPassVertex {
                position: [0.0, 1.0],
            },
        ];
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let rect_quad_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("PaintPass Rect Quad Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let rect_quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PaintPass Rect Quad Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let rect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PaintPass Rect Pipeline"),
            layout: Some(&rect_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &rect_shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<PaintPassVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    }],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &rect_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let text_data_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("PaintPass Text Data Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let text_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PaintPass Text Instance Buffer"),
            size: (std::mem::size_of::<TextInstanceData>() * INITIAL_TEXT_INSTANCE_CAPACITY) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let character_color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PaintPass Character Color Buffer"),
            size: (std::mem::size_of::<[f32; 4]>() * INITIAL_CHARACTER_COLOR_CAPACITY) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let character_bg_color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PaintPass Character BG Color Buffer"),
            size: (std::mem::size_of::<[f32; 4]>() * INITIAL_CHARACTER_COLOR_CAPACITY) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let text_data_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PaintPass Text Data Bind Group"),
            layout: &text_data_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: text_instance_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: character_color_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: character_bg_color_buffer.as_entire_binding(),
                },
            ],
        });

        let text_font_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("PaintPass Text Font Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let text_global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("PaintPass Text Global Layout"),
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

        let text_global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("PaintPass Text Global Bind Group"),
            layout: &text_global_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_uniform_buffer.as_entire_binding(),
            }],
        });

        let text_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "PaintPass Text Bitmap Shader",
            include_str!("../../../shaders/paint_text.wgsl"),
        );

        let text_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PaintPass Text Pipeline Layout"),
            bind_group_layouts: &[
                Some(&text_data_bind_group_layout),
                Some(&text_font_bind_group_layout),
                Some(&text_global_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let text_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PaintPass Text Vertex Buffer"),
            size: (std::mem::size_of::<UiTextVertex>() * INITIAL_TEXT_VERTEX_CAPACITY) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let text_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("PaintPass Text Index Buffer"),
            size: (std::mem::size_of::<u32>() * INITIAL_TEXT_INDEX_CAPACITY) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PaintPass Text Pipeline"),
            layout: Some(&text_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &text_shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<UiTextVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: (3 * 4) as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: (5 * 4) as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Uint32,
                        },
                        wgpu::VertexAttribute {
                            offset: (6 * 4) as wgpu::BufferAddress,
                            shader_location: 3,
                            format: wgpu::VertexFormat::Uint32,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &text_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            rect_pipeline,
            text_pipeline,
            global_uniform_buffer,
            global_uniform_bind_group,
            rect_quad_vertex_buffer,
            rect_quad_index_buffer,
            rect_instance_buffer,
            rect_draw_order_buffer,
            rect_instance_bind_group_layout,
            rect_instance_bind_group,
            rect_instance_capacity: INITIAL_RECT_CAPACITY,
            rect_draw_order_capacity: INITIAL_RECT_CAPACITY,
            rect_count: 0,
            prev_rect_instances: Vec::new(),
            text_vertex_buffer,
            text_index_buffer,
            text_instance_buffer,
            character_color_buffer,
            character_bg_color_buffer,
            text_data_bind_group_layout,
            text_data_bind_group,
            text_font_bind_group_layout,
            text_font_bind_groups: Vec::new(),
            _text_global_bind_group_layout: text_global_bind_group_layout,
            text_global_bind_group,
            prev_text_instance_data: Vec::new(),
            text_slots: std::collections::HashMap::new(),
            text_vertex_slab: super::SlabAllocator::default(),
            text_index_slab: super::SlabAllocator::default(),
            char_color_slab: super::SlabAllocator::default(),
            prev_character_colors: Vec::new(),
            prev_character_bg_colors: Vec::new(),
            text_vertex_capacity: INITIAL_TEXT_VERTEX_CAPACITY,
            text_index_capacity: INITIAL_TEXT_INDEX_CAPACITY,
            text_instance_capacity: INITIAL_TEXT_INSTANCE_CAPACITY,
            character_color_capacity: INITIAL_CHARACTER_COLOR_CAPACITY,
            character_bg_color_capacity: INITIAL_CHARACTER_COLOR_CAPACITY,
            text_draw_calls: Vec::new(),
            layer_draw_groups: Vec::new(),
            cached_atlas_view: None,
            screen_width: 800.0,
            screen_height: 600.0,
        }
    }
}
