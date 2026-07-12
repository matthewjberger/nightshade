use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};
use wgpu::util::DeviceExt;

const KERNEL_SIZE: usize = 64;
const NOISE_SIZE: u32 = 4;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct SsaoParams {
    projection: [[f32; 4]; 4],
    inv_projection: [[f32; 4]; 4],
    screen_size: [f32; 2],
    radius: f32,
    bias: f32,
    intensity: f32,
    enabled: f32,
    sample_count: f32,
    _padding: f32,
}

pub struct SsaoPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    kernel_buffer: wgpu::Buffer,
    params_buffer: wgpu::Buffer,
    noise_texture: wgpu::Texture,
    noise_view: wgpu::TextureView,
    point_sampler: wgpu::Sampler,
    noise_sampler: wgpu::Sampler,
    cached_bind_group: Option<wgpu::BindGroup>,
    cached_params: SsaoParams,
    noise_initialized: bool,
}

impl SsaoPass {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "ssao.wgsl",
            include_str!("../../shaders/ssao.wgsl"),
        );

        let kernel_samples = Self::generate_kernel();
        let kernel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SSAO Kernel Buffer"),
            contents: bytemuck::cast_slice(&kernel_samples),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let params = SsaoParams {
            projection: [[0.0; 4]; 4],
            inv_projection: [[0.0; 4]; 4],
            screen_size: [1920.0, 1080.0],
            radius: 0.5,
            bias: 0.025,
            intensity: 1.0,
            enabled: 1.0,
            sample_count: 64.0,
            _padding: 0.0,
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SSAO Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let noise_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO Noise Texture"),
            size: wgpu::Extent3d {
                width: NOISE_SIZE,
                height: NOISE_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let noise_view = noise_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let point_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let noise_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("SSAO Bind Group Layout"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SSAO Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("SSAO Pipeline"),
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
                    format: wgpu::TextureFormat::R8Unorm,
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
            kernel_buffer,
            params_buffer,
            noise_texture,
            noise_view,
            point_sampler,
            noise_sampler,
            cached_bind_group: None,
            cached_params: params,
            noise_initialized: false,
        }
    }

    fn generate_kernel() -> [[f32; 4]; KERNEL_SIZE] {
        let mut kernel = [[0.0f32; 4]; KERNEL_SIZE];
        let mut rng_state = 12345u32;

        for (index, sample) in kernel.iter_mut().enumerate() {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let x = (rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0;

            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let y = (rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0;

            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let z = (rng_state as f32 / u32::MAX as f32) * 0.9 + 0.1;

            let length = (x * x + y * y + z * z).sqrt();
            let (normalized_x, normalized_y, normalized_z) = if length > 0.001 {
                (x / length, y / length, z / length)
            } else {
                (0.0, 0.0, 1.0)
            };

            let scale = index as f32 / KERNEL_SIZE as f32;
            let scale = 0.1 + scale * scale * 0.9;

            *sample = [
                normalized_x * scale,
                normalized_y * scale,
                normalized_z * scale,
                0.0,
            ];
        }

        kernel
    }

    fn generate_noise() -> Vec<u8> {
        let mut noise = Vec::with_capacity((NOISE_SIZE * NOISE_SIZE * 4) as usize);
        let mut rng_state = 54321u32;

        for _ in 0..(NOISE_SIZE * NOISE_SIZE) {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let x = (rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0;

            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let y = (rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0;

            noise.push(((x * 0.5 + 0.5) * 255.0) as u8);
            noise.push(((y * 0.5 + 0.5) * 255.0) as u8);
            noise.push(128);
            noise.push(255);
        }

        noise
    }
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for SsaoPass {
    fn name(&self) -> &str {
        "ssao_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["depth", "view_normals"]
    }

    fn writes(&self) -> Vec<&str> {
        vec!["ssao_raw"]
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
        if !self.noise_initialized {
            let noise_data = Self::generate_noise();
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.noise_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &noise_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(NOISE_SIZE * 4),
                    rows_per_image: Some(NOISE_SIZE),
                },
                wgpu::Extent3d {
                    width: NOISE_SIZE,
                    height: NOISE_SIZE,
                    depth_or_array_layers: 1,
                },
            );
            self.noise_initialized = true;
        }

        let graphics = &configs.scene;
        let enabled = if graphics.active_view.ssao_enabled {
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

            let (screen_width, screen_height) = configs.view.viewport_size.unwrap_or((1920, 1080));

            let view = &graphics.active_view;
            let scale = graphics
                .adaptive_sampling
                .ssao_sample_scale
                .clamp(0.25, 1.0);
            let scaled_sample_count = ((view.ssao_sample_count as f32) * scale).round().max(4.0);
            let params = SsaoParams {
                projection,
                inv_projection,
                screen_size: [
                    (screen_width / 2).max(1) as f32,
                    (screen_height / 2).max(1) as f32,
                ],
                radius: view.ssao_radius,
                bias: view.ssao_bias,
                intensity: view.ssao_intensity,
                enabled,
                sample_count: scaled_sample_count,
                _padding: 0.0,
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
        if !context.configs.scene.active_view.ssao_enabled {
            return Ok(context.into_sub_graph_commands());
        }

        if self.cached_bind_group.is_none() {
            let (depth_view, _, _) = context.get_depth_attachment("depth")?;
            let normal_view = context.get_texture_view("view_normals")?;

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
                            resource: wgpu::BindingResource::TextureView(&self.noise_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&self.point_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&self.noise_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: self.params_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: self.kernel_buffer.as_entire_binding(),
                        },
                    ],
                    label: Some("SSAO Bind Group"),
                },
            ));
        }

        let (output_view, output_load_op, output_store_op) =
            context.get_color_attachment("ssao_raw")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("SSAO Render Pass"),
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
