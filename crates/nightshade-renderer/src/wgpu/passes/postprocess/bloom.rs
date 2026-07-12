use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};
use wgpu::util::DeviceExt;

const MIP_CHAIN_LENGTH: usize = 6;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BloomParams {
    filter_radius: f32,
    _padding0: f32,
    _padding1: f32,
    _padding2: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DownsampleParams {
    mip_level: u32,
    threshold: f32,
    knee: f32,
    exposure: f32,
}

struct BloomMip {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
}

const MIP_CHAIN_CACHE_CAPACITY: usize = 4;

pub struct BloomPass {
    downsample_pipeline: wgpu::RenderPipeline,
    upsample_pipeline: wgpu::RenderPipeline,
    downsample_bind_group_layout: wgpu::BindGroupLayout,
    upsample_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    downsample_params_buffers: Vec<wgpu::Buffer>,
    mip_chain: Vec<BloomMip>,
    mip_chain_cache: Vec<((u32, u32), Vec<BloomMip>)>,
    downsample_bind_groups: Vec<wgpu::BindGroup>,
    upsample_bind_groups: Vec<wgpu::BindGroup>,
    current_size: (u32, u32),
    threshold: f32,
    knee: f32,
    filter_radius: f32,
    exposure: f32,
}

impl BloomPass {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let downsample_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "bloom_downsample.wgsl",
            include_str!("../../shaders/bloom_downsample.wgsl"),
        );
        let upsample_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "bloom_upsample.wgsl",
            include_str!("../../shaders/bloom_upsample.wgsl"),
        );

        let params = BloomParams {
            filter_radius: 0.005,
            _padding0: 0.0,
            _padding1: 0.0,
            _padding2: 0.0,
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bloom Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let default_threshold = 1.0_f32;
        let default_knee = 0.5_f32;
        let default_filter_radius = 0.005_f32;
        let mut downsample_params_buffers = Vec::with_capacity(MIP_CHAIN_LENGTH);
        for mip_level in 0..MIP_CHAIN_LENGTH {
            let downsample_params = DownsampleParams {
                mip_level: mip_level as u32,
                threshold: default_threshold,
                knee: default_knee,
                exposure: 1.0,
            };
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Bloom Downsample Params Buffer {}", mip_level)),
                contents: bytemuck::cast_slice(&[downsample_params]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            downsample_params_buffers.push(buffer);
        }

        let downsample_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bloom Downsample Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
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

        let upsample_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bloom Upsample Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
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

        let downsample_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Bloom Downsample Pipeline Layout"),
                bind_group_layouts: &[Some(&downsample_bind_group_layout)],
                immediate_size: 0,
            });

        let upsample_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Bloom Upsample Pipeline Layout"),
                bind_group_layouts: &[Some(&upsample_bind_group_layout)],
                immediate_size: 0,
            });

        let downsample_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Bloom Downsample Pipeline"),
            layout: Some(&downsample_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &downsample_shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &downsample_shader,
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

        let upsample_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Bloom Upsample Pipeline"),
            layout: Some(&upsample_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &upsample_shader,
                entry_point: Some("vertex_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &upsample_shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let mut mip_chain = Vec::with_capacity(MIP_CHAIN_LENGTH);
        let mut mip_width = width / 2;
        let mut mip_height = height / 2;

        for mip_level in 0..MIP_CHAIN_LENGTH {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("Bloom Mip {}", mip_level)),
                size: wgpu::Extent3d {
                    width: mip_width.max(1),
                    height: mip_height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            mip_chain.push(BloomMip {
                _texture: texture,
                view,
            });

            mip_width /= 2;
            mip_height /= 2;
        }

        Self {
            downsample_pipeline,
            upsample_pipeline,
            downsample_bind_group_layout,
            upsample_bind_group_layout,
            sampler,
            params_buffer,
            downsample_params_buffers,
            mip_chain,
            mip_chain_cache: Vec::new(),
            downsample_bind_groups: Vec::new(),
            upsample_bind_groups: Vec::new(),
            current_size: (width, height),
            threshold: default_threshold,
            knee: default_knee,
            filter_radius: default_filter_radius,
            exposure: 1.0,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) {
        if self.current_size == (width, height) {
            return;
        }

        let params = BloomParams {
            filter_radius: self.filter_radius,
            _padding0: 0.0,
            _padding1: 0.0,
            _padding2: 0.0,
        };

        queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[params]));

        self.downsample_bind_groups.clear();
        self.upsample_bind_groups.clear();

        if let Some(position) = self
            .mip_chain_cache
            .iter()
            .position(|(size, _)| *size == (width, height))
        {
            let (cached_size, cached_chain) = self.mip_chain_cache.remove(position);
            let previous_chain = std::mem::replace(&mut self.mip_chain, cached_chain);
            self.push_mip_chain_cache_entry(self.current_size, previous_chain);
            self.current_size = cached_size;
            return;
        }

        let new_chain = BloomPass::create_mip_chain(device, width, height);
        let previous_size = self.current_size;
        let previous_chain = std::mem::replace(&mut self.mip_chain, new_chain);
        self.push_mip_chain_cache_entry(previous_size, previous_chain);
        self.current_size = (width, height);
    }

    fn create_mip_chain(device: &wgpu::Device, width: u32, height: u32) -> Vec<BloomMip> {
        let mut chain = Vec::with_capacity(MIP_CHAIN_LENGTH);
        let mut mip_width = width / 2;
        let mut mip_height = height / 2;

        for mip_level in 0..MIP_CHAIN_LENGTH {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("Bloom Mip {}", mip_level)),
                size: wgpu::Extent3d {
                    width: mip_width.max(1),
                    height: mip_height.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            chain.push(BloomMip {
                _texture: texture,
                view,
            });

            mip_width /= 2;
            mip_height /= 2;
        }

        chain
    }

    fn push_mip_chain_cache_entry(&mut self, size: (u32, u32), chain: Vec<BloomMip>) {
        if size.0 == 0 || size.1 == 0 || chain.is_empty() {
            return;
        }
        self.mip_chain_cache.push((size, chain));
        if self.mip_chain_cache.len() > MIP_CHAIN_CACHE_CAPACITY {
            self.mip_chain_cache.remove(0);
        }
    }
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for BloomPass {
    fn name(&self) -> &str {
        "bloom_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["hdr"]
    }

    fn writes(&self) -> Vec<&str> {
        vec!["bloom"]
    }

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        let view = &configs.scene.active_view;
        let new_threshold = view.bloom_threshold;
        let new_knee = view.bloom_knee.max(0.0);
        let new_filter_radius = view.bloom_filter_radius.max(0.0);
        let grading = &configs.settings.color_grading;
        let new_exposure =
            (grading.exposure * 2.0_f32.powf(grading.exposure_compensation_ev)).max(0.0);
        let threshold_changed = (new_threshold - self.threshold).abs() > f32::EPSILON;
        let knee_changed = (new_knee - self.knee).abs() > f32::EPSILON;
        let exposure_changed = (new_exposure - self.exposure).abs() > f32::EPSILON;
        if threshold_changed || knee_changed || exposure_changed {
            self.threshold = new_threshold;
            self.knee = new_knee;
            self.exposure = new_exposure;
            for (mip_level, buffer) in self.downsample_params_buffers.iter().enumerate() {
                let params = DownsampleParams {
                    mip_level: mip_level as u32,
                    threshold: self.threshold,
                    knee: self.knee,
                    exposure: self.exposure,
                };
                queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[params]));
            }
        }
        if (new_filter_radius - self.filter_radius).abs() > f32::EPSILON {
            self.filter_radius = new_filter_radius;
            let params = BloomParams {
                filter_radius: self.filter_radius,
                _padding0: 0.0,
                _padding1: 0.0,
                _padding2: 0.0,
            };
            queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[params]));
        }
    }

    fn invalidate_bind_groups(&mut self) {
        self.downsample_bind_groups.clear();
        self.upsample_bind_groups.clear();
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::wgpu::render_configs::RenderInputs>,
    ) -> crate::wgpu::rendergraph::Result<Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>>
    {
        if !context.is_pass_enabled() {
            return Ok(context.into_sub_graph_commands());
        }
        if !context.configs.scene.active_view.bloom_enabled {
            return Ok(context.into_sub_graph_commands());
        }

        if self.downsample_bind_groups.is_empty() {
            let hdr_view = context.get_texture_view("hdr")?;

            let first_bind_group = context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.downsample_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(hdr_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.downsample_params_buffers[0].as_entire_binding(),
                        },
                    ],
                    label: Some("Bloom Downsample Bind Group 0"),
                });
            self.downsample_bind_groups.push(first_bind_group);

            for index in 1..MIP_CHAIN_LENGTH {
                let bind_group = context
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &self.downsample_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(
                                    &self.mip_chain[index - 1].view,
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: self.downsample_params_buffers[index].as_entire_binding(),
                            },
                        ],
                        label: Some(&format!("Bloom Downsample Bind Group {}", index)),
                    });
                self.downsample_bind_groups.push(bind_group);
            }
        }

        if self.upsample_bind_groups.is_empty() {
            for index in 0..(MIP_CHAIN_LENGTH - 1) {
                let source_mip = MIP_CHAIN_LENGTH - 1 - index;

                let bind_group = context
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &self.upsample_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(
                                    &self.mip_chain[source_mip].view,
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: self.params_buffer.as_entire_binding(),
                            },
                        ],
                        label: Some(&format!("Bloom Upsample Bind Group {}", index)),
                    });
                self.upsample_bind_groups.push(bind_group);
            }
        }

        for index in 0..MIP_CHAIN_LENGTH {
            let mut render_pass = context
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some(&format!("Bloom Downsample Pass {}", index)),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.mip_chain[index].view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

            render_pass.set_pipeline(&self.downsample_pipeline);
            render_pass.set_bind_group(0, &self.downsample_bind_groups[index], &[]);
            render_pass.draw(0..3, 0..1);
        }

        for index in 0..(MIP_CHAIN_LENGTH - 1) {
            let target_mip = MIP_CHAIN_LENGTH - 2 - index;
            let is_final = target_mip == 0;

            if is_final {
                let (bloom_view, bloom_load_op, bloom_store_op) =
                    context.get_color_attachment("bloom")?;

                let mut render_pass =
                    context
                        .encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some(&format!("Bloom Upsample Pass {} (Final)", index)),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: bloom_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: bloom_load_op,
                                    store: bloom_store_op,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });

                render_pass.set_pipeline(&self.upsample_pipeline);
                render_pass.set_bind_group(0, &self.upsample_bind_groups[index], &[]);
                render_pass.draw(0..3, 0..1);
            } else {
                let mut render_pass =
                    context
                        .encoder
                        .begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some(&format!("Bloom Upsample Pass {}", index)),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &self.mip_chain[target_mip].view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        });

                render_pass.set_pipeline(&self.upsample_pipeline);
                render_pass.set_bind_group(0, &self.upsample_bind_groups[index], &[]);
                render_pass.draw(0..3, 0..1);
            }
        }

        Ok(context.into_sub_graph_commands())
    }
}
