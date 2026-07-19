use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct SsrParams {
    projection: [[f32; 4]; 4],
    inv_projection: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    inv_view: [[f32; 4]; 4],
    screen_size: [f32; 2],
    max_steps: u32,
    thickness: f32,
    max_distance: f32,
    stride: f32,
    fade_start: f32,
    fade_end: f32,
    intensity: f32,
    enabled: f32,
    _padding: [f32; 2],
}

/// Screen-space reflections pass. Ray-marches the half-resolution depth and
/// normal buffers against the scene color to produce a raw reflection target.
pub struct SsrPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    params_buffer: wgpu::Buffer,
    point_sampler: wgpu::Sampler,
    linear_sampler: wgpu::Sampler,
    cached_bind_group: Option<wgpu::BindGroup>,
    cached_params: SsrParams,
}

impl SsrPass {
    /// Compiles the SSR shader and builds the pipeline, samplers, and parameter
    /// buffer.
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "ssr.wgsl",
            include_str!("../../shaders/ssr.wgsl"),
        );

        let params = SsrParams {
            projection: [[0.0; 4]; 4],
            inv_projection: [[0.0; 4]; 4],
            view: [[0.0; 4]; 4],
            inv_view: [[0.0; 4]; 4],
            screen_size: [1920.0, 1080.0],
            max_steps: 64,
            thickness: 0.3,
            max_distance: 50.0,
            stride: 1.0,
            fade_start: 0.8,
            fade_end: 1.0,
            intensity: 1.0,
            enabled: 0.0,
            _padding: [0.0; 2],
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SSR Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let point_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SSR Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SSR Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SSR Pipeline"),
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
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: None,
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            params_buffer,
            point_sampler,
            linear_sampler,
            cached_bind_group: None,
            cached_params: params,
        }
    }
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for SsrPass {
    fn name(&self) -> &str {
        "ssr_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["depth", "view_normals", "scene_color"]
    }

    fn writes(&self) -> Vec<&str> {
        vec!["ssr_raw"]
    }

    fn invalidate_bind_groups(&mut self) {
        self.cached_bind_group = None;
    }

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        let graphics = &configs.scene;
        let enabled = if graphics.active_view.ssr_enabled {
            1.0
        } else {
            0.0
        };

        if let Some(camera_matrices) = configs.scene.render_view.as_ref() {
            let projection: [[f32; 4]; 4] = camera_matrices.projection.into();
            let inv_projection: [[f32; 4]; 4] = camera_matrices
                .projection
                .try_inverse()
                .unwrap_or(nalgebra_glm::Mat4::identity())
                .into();
            let view: [[f32; 4]; 4] = camera_matrices.view.into();
            let inv_view: [[f32; 4]; 4] = camera_matrices
                .view
                .try_inverse()
                .unwrap_or(nalgebra_glm::Mat4::identity())
                .into();

            let (screen_width, screen_height) = configs.view.viewport_size.unwrap_or((1920, 1080));

            let active = &graphics.active_view;
            let params = SsrParams {
                projection,
                inv_projection,
                view,
                inv_view,
                screen_size: [
                    (screen_width / 2).max(1) as f32,
                    (screen_height / 2).max(1) as f32,
                ],
                max_steps: active.ssr_max_steps,
                thickness: active.ssr_thickness,
                max_distance: active.ssr_max_distance,
                stride: active.ssr_stride,
                fade_start: active.ssr_fade_start,
                fade_end: active.ssr_fade_end,
                intensity: active.ssr_intensity,
                enabled,
                _padding: [0.0; 2],
            };

            if params != self.cached_params {
                queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[params]));
                self.cached_params = params;
            }
        }
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::wgpu::render_configs::RenderInputs>,
    ) -> crate::wgpu::rendergraph::Result<Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>>
    {
        if !context.is_pass_enabled() {
            return Ok(context.into_sub_graph_commands());
        }
        if !context.configs.scene.active_view.ssr_enabled {
            return Ok(context.into_sub_graph_commands());
        }

        if self.cached_bind_group.is_none() {
            let (depth_view, _, _) = context.get_depth_attachment("depth")?;
            let normal_view = context.get_texture_view("view_normals")?;
            let scene_color_view = context.get_texture_view("scene_color")?;

            self.cached_bind_group = Some(context.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(depth_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(normal_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(scene_color_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&self.point_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&self.linear_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: self.params_buffer.as_entire_binding(),
                        },
                    ],
                    label: Some("SSR Bind Group"),
                },
            ));
        }

        let (output_view, output_load_op, output_store_op) =
            context.get_color_attachment("ssr_raw")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SSR Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: output_load_op,
                        store: output_store_op,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, self.cached_bind_group.as_ref().unwrap(), &[]);
        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}
