use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};

pub struct GridPass {
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl GridPass {
    pub fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Uniform Buffer"),
            size: std::mem::size_of::<GridUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Grid Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Grid Bind Group"),
        });

        let shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "grid.wgsl",
            include_str!("../../shaders/grid.wgsl"),
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Grid Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
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
                format: depth_format,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: -2,
                    slope_scale: -2.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            uniform_buffer,
            bind_group,
            pipeline,
        }
    }

    pub fn record(
        &self,
        render_pass: &mut wgpu::RenderPass<'_>,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        if !configs.debug_draw.show_grid {
            return;
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for GridPass {
    fn name(&self) -> &str {
        "grid_pass"
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
            let (is_orthographic, orthographic_scale) =
                if let Some((x_mag, y_mag)) = camera_matrices.orthographic {
                    let avg_mag = (x_mag + y_mag) * 0.5;
                    let scale = if avg_mag > 100.0 {
                        avg_mag * 0.001
                    } else {
                        avg_mag * 0.1
                    };
                    (true, scale)
                } else {
                    (false, 1.0)
                };

            let uniform = GridUniform::new(
                camera_matrices.projection,
                camera_matrices.view,
                camera_matrices.camera_position,
                is_orthographic,
                orthographic_scale,
            );

            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
        }
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::wgpu::render_configs::RenderInputs>,
    ) -> crate::wgpu::rendergraph::Result<Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>>
    {
        if !context.configs.debug_draw.show_grid {
            return Ok(context.into_sub_graph_commands());
        }

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Grid Pass"),
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

        self.record(&mut render_pass, context.configs);

        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridUniform {
    view_proj: [[f32; 4]; 4],
    camera_world_pos: [f32; 3],
    grid_size: f32,
    grid_min_pixels: f32,
    grid_cell_size: f32,
    orthographic_scale: f32,
    is_orthographic: f32,
}

impl GridUniform {
    pub fn new(
        projection: nalgebra_glm::Mat4,
        view: nalgebra_glm::Mat4,
        camera_position: nalgebra_glm::Vec3,
        is_orthographic: bool,
        orthographic_scale: f32,
    ) -> Self {
        let view_proj: [[f32; 4]; 4] = (projection * view).into();

        Self {
            view_proj,
            camera_world_pos: [camera_position.x, camera_position.y, camera_position.z],
            grid_size: 500.0,
            grid_min_pixels: 2.0,
            grid_cell_size: 1.0,
            orthographic_scale,
            is_orthographic: if is_orthographic { 1.0 } else { 0.0 },
        }
    }
}
