//! The final postprocess pass: tonemapping, color grading, bloom, ambient
//! occlusion, auto exposure, and the color-grading LUT composited to the
//! output.

use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PostProcessParams {
    bloom_enabled: f32,
    bloom_intensity: f32,
    letterbox_amount: f32,
    gamma: f32,
    saturation: f32,
    brightness: f32,
    contrast: f32,
    tonemap_algorithm: u32,
    ssao_enabled: f32,
    texture_debug_stripes: u32,
    time: f32,
    texture_debug_stripes_speed: f32,
    ssgi_enabled: f32,
    ssgi_intensity: f32,
    ssao_visualization: f32,
    ssr_enabled: f32,
    exposure: f32,
    auto_exposure_enabled: u32,
    auto_exposure_target: f32,
    exposure_compensation_scale: f32,
    auto_exposure_min_scale: f32,
    auto_exposure_max_scale: f32,
    vibrance: f32,
    vignette_intensity: f32,
    vignette_radius: f32,
    vignette_smoothness: f32,
    chromatic_aberration: f32,
    color_lut_weight: f32,
}

/// The fullscreen postprocess pass compositing the HDR scene to the output.
pub struct PostProcessPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    cached_bind_group: Option<wgpu::BindGroup>,
    hdr_source_is_dof_output: bool,
    bloom_intensity: f32,
    letterbox_amount: f32,
    fallback_texture_view: wgpu::TextureView,
    auto_exposure_pipeline: wgpu::ComputePipeline,
    auto_exposure_bind_group_layout: wgpu::BindGroupLayout,
    auto_exposure_bind_group: Option<wgpu::BindGroup>,
    auto_exposure_buffer: wgpu::Buffer,
    color_lut_texture: wgpu::Texture,
    color_lut_view: wgpu::TextureView,
    color_lut_sampler: wgpu::Sampler,
    pending_color_lut: Option<Vec<u8>>,
}

const COLOR_LUT_SIZE: u32 = 16;

fn identity_color_lut_data() -> Vec<u8> {
    let size = COLOR_LUT_SIZE as usize;
    let mut data = Vec::with_capacity(size * size * size * 4);
    for b in 0..size {
        for g in 0..size {
            for r in 0..size {
                let scale = 255.0 / (size as f32 - 1.0);
                data.push((r as f32 * scale).round() as u8);
                data.push((g as f32 * scale).round() as u8);
                data.push((b as f32 * scale).round() as u8);
                data.push(255);
            }
        }
    }
    data
}

fn upload_color_lut(queue: &wgpu::Queue, texture: &wgpu::Texture, data: &[u8]) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * COLOR_LUT_SIZE),
            rows_per_image: Some(COLOR_LUT_SIZE),
        },
        wgpu::Extent3d {
            width: COLOR_LUT_SIZE,
            height: COLOR_LUT_SIZE,
            depth_or_array_layers: COLOR_LUT_SIZE,
        },
    );
}

impl PostProcessPass {
    /// Builds the pass targeting `surface_format` with the given starting
    /// bloom intensity, seeding an identity color-grading LUT.
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        bloom_intensity: f32,
    ) -> Self {
        let color_lut_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Color Grading LUT"),
            size: wgpu::Extent3d {
                width: COLOR_LUT_SIZE,
                height: COLOR_LUT_SIZE,
                depth_or_array_layers: COLOR_LUT_SIZE,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let color_lut_view = color_lut_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let color_lut_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Color LUT Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "postprocess.wgsl",
            include_str!("../../shaders/postprocess.wgsl"),
        );

        let params = PostProcessParams {
            bloom_enabled: 0.0,
            bloom_intensity,
            letterbox_amount: 0.0,
            gamma: 2.2,
            saturation: 1.0,
            brightness: 0.0,
            contrast: 1.0,
            tonemap_algorithm: 0,
            ssao_enabled: 0.0,
            texture_debug_stripes: 0,
            time: 0.0,
            texture_debug_stripes_speed: 100.0,
            ssgi_enabled: 0.0,
            ssgi_intensity: 1.0,
            ssao_visualization: 0.0,
            ssr_enabled: 0.0,
            exposure: 1.0,
            auto_exposure_enabled: 0,
            auto_exposure_target: 0.18,
            exposure_compensation_scale: 1.0,
            auto_exposure_min_scale: (-3.0_f32).exp2(),
            auto_exposure_max_scale: 5.0_f32.exp2(),
            vibrance: 0.0,
            vignette_intensity: 0.0,
            vignette_radius: 0.0,
            vignette_smoothness: 0.0,
            chromatic_aberration: 0.0,
            color_lut_weight: 0.0,
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("PostProcess Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let fallback_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("PostProcess Fallback Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let fallback_texture_view =
            fallback_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("PostProcess Bind Group Layout"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
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
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 11,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 12,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 13,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("PostProcess Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("PostProcess Pipeline"),
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
                    format: surface_format,
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let auto_exposure_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "auto_exposure.wgsl",
            include_str!("../../shaders/auto_exposure.wgsl"),
        );

        let auto_exposure_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Auto Exposure Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
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
                ],
            });

        let auto_exposure_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Auto Exposure Pipeline Layout"),
                bind_group_layouts: &[Some(&auto_exposure_bind_group_layout)],
                immediate_size: 0,
            });

        let auto_exposure_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Auto Exposure Pipeline"),
                layout: Some(&auto_exposure_pipeline_layout),
                module: &auto_exposure_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        let auto_exposure_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Auto Exposure Buffer"),
            // Layout: target, current, rate, dt, primed, pad, pad, pad (8 × 4 bytes)
            contents: bytemuck::cast_slice(&[
                0.0_f32, 0.0_f32, 1.0_f32, 0.0_f32, 0.0_f32, 0.0, 0.0, 0.0,
            ]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            params_buffer,
            cached_bind_group: None,
            hdr_source_is_dof_output: false,
            bloom_intensity,
            letterbox_amount: 0.0,
            fallback_texture_view,
            auto_exposure_pipeline,
            auto_exposure_bind_group_layout,
            auto_exposure_bind_group: None,
            auto_exposure_buffer,
            color_lut_texture,
            color_lut_view,
            color_lut_sampler,
            pending_color_lut: Some(identity_color_lut_data()),
        }
    }

    /// Queues raw RGBA8 LUT pixels to upload on the next frame.
    pub fn set_color_lut(&mut self, data: Vec<u8>) {
        self.pending_color_lut = Some(data);
    }

    /// Sets the bloom intensity applied by the pass.
    pub fn set_bloom_intensity(&mut self, intensity: f32) {
        self.bloom_intensity = intensity;
    }
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for PostProcessPass {
    fn name(&self) -> &str {
        "postprocess_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["hdr", "scene_color", "bloom", "ssao"]
    }

    fn writes(&self) -> Vec<&str> {
        vec!["output"]
    }

    fn optional_reads(&self) -> Vec<&str> {
        vec!["ssgi", "ssr"]
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
        self.letterbox_amount = configs.scene.letterbox_amount;
        let view = &configs.scene.active_view;
        let color_grading = &view.color_grading;
        let bloom_enabled = if view.bloom_enabled { 1.0 } else { 0.0 };
        let ssao_enabled = if view.ssao_enabled { 1.0 } else { 0.0 };
        let bloom_intensity = view.bloom_intensity;
        let texture_debug_stripes = if configs.debug_draw.texture_debug_stripes {
            1
        } else {
            0
        };
        let ssgi_enabled = if view.ssgi_enabled { 1.0 } else { 0.0 };
        let ssgi_intensity = view.ssgi_intensity;
        let ssao_visualization = if view.ssao_visualization { 1.0 } else { 0.0 };
        let ssr_enabled = if view.ssr_enabled { 1.0 } else { 0.0 };
        let min_ev = color_grading
            .auto_exposure_min_ev
            .min(color_grading.auto_exposure_max_ev);
        let max_ev = color_grading
            .auto_exposure_min_ev
            .max(color_grading.auto_exposure_max_ev);
        let params = PostProcessParams {
            bloom_enabled,
            bloom_intensity,
            letterbox_amount: self.letterbox_amount,
            gamma: color_grading.gamma,
            saturation: color_grading.saturation,
            brightness: color_grading.brightness,
            contrast: color_grading.contrast,
            tonemap_algorithm: color_grading.tonemap_algorithm.as_u32(),
            ssao_enabled,
            texture_debug_stripes,
            time: configs.view.uptime_milliseconds as f32 / 1000.0,
            texture_debug_stripes_speed: configs.debug_draw.texture_debug_stripes_speed,
            ssgi_enabled,
            ssgi_intensity,
            ssao_visualization,
            ssr_enabled,
            exposure: color_grading.exposure,
            auto_exposure_enabled: u32::from(color_grading.auto_exposure),
            auto_exposure_target: color_grading.auto_exposure_target,
            exposure_compensation_scale: color_grading.exposure_compensation_ev.exp2(),
            auto_exposure_min_scale: min_ev.exp2(),
            auto_exposure_max_scale: max_ev.exp2(),
            vibrance: color_grading.vibrance,
            vignette_intensity: color_grading.vignette_intensity,
            vignette_radius: color_grading.vignette_radius,
            vignette_smoothness: color_grading.vignette_smoothness,
            chromatic_aberration: color_grading.chromatic_aberration,
            color_lut_weight: color_grading.color_lut_weight,
        };
        if let Some(data) = self.pending_color_lut.take() {
            upload_color_lut(queue, &self.color_lut_texture, &data);
        }
        queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[params]));
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::wgpu::render_configs::RenderInputs>,
    ) -> crate::wgpu::rendergraph::Result<Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>>
    {
        let hdr_source_is_dof_output = context.configs.scene.active_view.depth_of_field.enabled;
        if self.hdr_source_is_dof_output != hdr_source_is_dof_output {
            self.cached_bind_group = None;
            self.hdr_source_is_dof_output = hdr_source_is_dof_output;
        }

        if self.cached_bind_group.is_none() {
            let hdr_view = if hdr_source_is_dof_output {
                context.get_texture_view("hdr")?
            } else {
                context.get_texture_view("scene_color")?
            };
            let bloom_view = context.get_texture_view("bloom")?;
            let ssao_view = context.get_texture_view("ssao")?;
            let ssgi_view = context
                .get_texture_view("ssgi")
                .unwrap_or(&self.fallback_texture_view);
            let ssr_view = context
                .get_texture_view("ssr")
                .unwrap_or(&self.fallback_texture_view);

            self.cached_bind_group = Some(context.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &self.bind_group_layout,
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
                            resource: self.params_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(bloom_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::TextureView(ssao_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 7,
                            resource: wgpu::BindingResource::TextureView(ssgi_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 8,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 9,
                            resource: wgpu::BindingResource::TextureView(ssr_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 10,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 11,
                            resource: self.auto_exposure_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 12,
                            resource: wgpu::BindingResource::TextureView(&self.color_lut_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 13,
                            resource: wgpu::BindingResource::Sampler(&self.color_lut_sampler),
                        },
                    ],
                    label: Some("PostProcess Bind Group"),
                },
            ));

            self.auto_exposure_bind_group = Some(context.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &self.auto_exposure_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(hdr_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.auto_exposure_buffer.as_entire_binding(),
                        },
                    ],
                    label: Some("Auto Exposure Bind Group"),
                },
            ));
        }

        if context.configs.settings.color_grading.auto_exposure
            && let Some(bg) = self.auto_exposure_bind_group.as_ref()
        {
            let adaptation_rate = context.configs.settings.color_grading.auto_exposure_rate;
            let delta_time = context.configs.view.delta_time.clamp(1.0e-4, 1.0);
            // Write adaptation_rate at offset 8 and delta_time at offset 12 (4-byte floats).
            context.queue.write_buffer(
                &self.auto_exposure_buffer,
                8,
                bytemuck::cast_slice(&[adaptation_rate, delta_time]),
            );

            let mut compute_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Auto Exposure Compute Pass"),
                        timestamp_writes: None,
                    });
            compute_pass.set_pipeline(&self.auto_exposure_pipeline);
            compute_pass.set_bind_group(0, bg, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1);
        }

        let (color_view, color_load_op, color_store_op) = context.get_color_attachment("output")?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("PostProcess Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: color_load_op,
                        store: color_store_op,
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
