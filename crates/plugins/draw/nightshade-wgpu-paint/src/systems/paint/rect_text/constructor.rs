use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;
use wgpu::util::DeviceExt;

use super::{
    GlobalUniforms, INITIAL_CHARACTER_COLOR_CAPACITY, INITIAL_RECT_CAPACITY,
    INITIAL_TEXT_INDEX_CAPACITY, INITIAL_TEXT_INSTANCE_CAPACITY, INITIAL_TEXT_VERTEX_CAPACITY,
    PaintPass, PaintPassVertex, PaintRectInstance, TextInstanceData, UiTextVertex,
};

impl PaintPass {
    pub fn new(build: PassBuild, color_format: wgpu::TextureFormat) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let global_uniform_bind_group_layout = build::bind_layout(
            device,
            "PaintPass Global Uniform Layout",
            &[build::uniform_at(0, wgpu::ShaderStages::VERTEX)],
        );

        let global_uniform_buffer = build::uniform(
            device,
            "PaintPass Global Uniform Buffer",
            std::mem::size_of::<GlobalUniforms>() as u64,
        );

        let global_uniform_bind_group = build::binding(
            device,
            "PaintPass Global Uniform Bind Group",
            &global_uniform_bind_group_layout,
            vec![global_uniform_buffer.as_entire_binding()],
        );

        let rect_instance_bind_group_layout = build::bind_layout(
            device,
            "PaintPass Rect Instance Layout",
            &[
                build::storage_at(0, false, wgpu::ShaderStages::VERTEX_FRAGMENT),
                build::storage_at(1, false, wgpu::ShaderStages::VERTEX),
            ],
        );

        let rect_instance_buffer = build::storage(
            device,
            "PaintPass Rect Instance Buffer",
            (std::mem::size_of::<PaintRectInstance>() * INITIAL_RECT_CAPACITY) as u64,
        );

        let rect_draw_order_buffer = build::storage(
            device,
            "PaintPass Rect Draw Order Buffer",
            (std::mem::size_of::<u32>() * INITIAL_RECT_CAPACITY) as u64,
        );

        let rect_instance_bind_group = build::binding(
            device,
            "PaintPass Rect Instance Bind Group",
            &rect_instance_bind_group_layout,
            vec![
                rect_instance_buffer.as_entire_binding(),
                rect_draw_order_buffer.as_entire_binding(),
            ],
        );

        let rect_shader = composer.compile_wgsl(
            device,
            "PaintPass Rect Shader",
            include_str!("../../../shaders/paint_rect.wgsl"),
        );

        let rect_pipeline_layout = build::pipeline_layout(
            device,
            "PaintPass Rect Pipeline Layout",
            &[
                &global_uniform_bind_group_layout,
                &rect_instance_bind_group_layout,
            ],
        );

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

        let rect_pipeline = build::Pipe::new(device, "PaintPass Rect Pipeline", &rect_shader)
            .layout(&rect_pipeline_layout)
            .buffers(&[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<PaintPassVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            }])
            .target(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })
            .depth(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
            .vertex("vs_main")
            .fragment("fs_main")
            .make();

        let text_data_bind_group_layout = build::bind_layout(
            device,
            "PaintPass Text Data Layout",
            &[
                build::storage_at(0, false, wgpu::ShaderStages::VERTEX_FRAGMENT),
                build::storage_at(1, false, wgpu::ShaderStages::FRAGMENT),
                build::storage_at(2, false, wgpu::ShaderStages::FRAGMENT),
            ],
        );

        let text_instance_buffer = build::storage(
            device,
            "PaintPass Text Instance Buffer",
            (std::mem::size_of::<TextInstanceData>() * INITIAL_TEXT_INSTANCE_CAPACITY) as u64,
        );

        let character_color_buffer = build::buffer(
            device,
            "PaintPass Character Color Buffer",
            (std::mem::size_of::<[f32; 4]>() * INITIAL_CHARACTER_COLOR_CAPACITY) as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        let character_bg_color_buffer = build::buffer(
            device,
            "PaintPass Character BG Color Buffer",
            (std::mem::size_of::<[f32; 4]>() * INITIAL_CHARACTER_COLOR_CAPACITY) as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        let text_data_bind_group = build::binding(
            device,
            "PaintPass Text Data Bind Group",
            &text_data_bind_group_layout,
            vec![
                text_instance_buffer.as_entire_binding(),
                character_color_buffer.as_entire_binding(),
                character_bg_color_buffer.as_entire_binding(),
            ],
        );

        let text_font_bind_group_layout = build::bind_layout(
            device,
            "PaintPass Text Font Layout",
            &[build::painted_at(0), build::filtering_at(1)],
        );

        let text_global_bind_group_layout = build::bind_layout(
            device,
            "PaintPass Text Global Layout",
            &[build::uniform_at(0, wgpu::ShaderStages::VERTEX)],
        );

        let text_global_bind_group = build::binding(
            device,
            "PaintPass Text Global Bind Group",
            &text_global_bind_group_layout,
            vec![global_uniform_buffer.as_entire_binding()],
        );

        let text_shader = composer.compile_wgsl(
            device,
            "PaintPass Text Bitmap Shader",
            include_str!("../../../shaders/paint_text.wgsl"),
        );

        let text_pipeline_layout = build::pipeline_layout(
            device,
            "PaintPass Text Pipeline Layout",
            &[
                &text_data_bind_group_layout,
                &text_font_bind_group_layout,
                &text_global_bind_group_layout,
            ],
        );

        let text_vertex_buffer = build::buffer(
            device,
            "PaintPass Text Vertex Buffer",
            (std::mem::size_of::<UiTextVertex>() * INITIAL_TEXT_VERTEX_CAPACITY) as u64,
            wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );

        let text_index_buffer = build::buffer(
            device,
            "PaintPass Text Index Buffer",
            (std::mem::size_of::<u32>() * INITIAL_TEXT_INDEX_CAPACITY) as u64,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        );

        let text_pipeline = build::Pipe::new(device, "PaintPass Text Pipeline", &text_shader)
            .layout(&text_pipeline_layout)
            .buffers(&[wgpu::VertexBufferLayout {
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
                    wgpu::VertexAttribute {
                        offset: (7 * 4) as wgpu::BufferAddress,
                        shader_location: 4,
                        format: wgpu::VertexFormat::Uint32,
                    },
                ],
            }])
            .target(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })
            .depth(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
            .vertex("vs_main")
            .fragment("fs_main")
            .make();

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
            draw_list: nightshade_paint::PaintDrawList::default(),
        }
    }
}
