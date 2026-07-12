use crate::config::Atmosphere;
#[cfg(feature = "assets")]
use crate::wgpu::envmap_filter::filter_environment_map;
#[cfg(feature = "assets")]
use crate::wgpu::hdr::generate_cubemap_mipmaps;
use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};

#[cfg(feature = "assets")]
const PROCEDURAL_CUBEMAP_SIZE: u32 = 1024;

#[cfg(feature = "assets")]
fn calculate_mip_count(size: u32) -> u32 {
    (size as f32).log2().floor() as u32 + 1
}

fn create_dummy_cubemap(device: &wgpu::Device) -> (wgpu::Texture, wgpu::TextureView) {
    let cubemap = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Dummy Cubemap Texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let cubemap_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    });

    (cubemap, cubemap_view)
}

pub struct SkyPass {
    uniform_buffer: wgpu::Buffer,
    procedural_bind_group: wgpu::BindGroup,
    sky_pipeline: wgpu::RenderPipeline,
    cloudy_sky_pipeline: wgpu::RenderPipeline,
    space_pipeline: wgpu::RenderPipeline,
    nebula_pipeline: wgpu::RenderPipeline,
    sunset_pipeline: wgpu::RenderPipeline,
    day_night_pipeline: wgpu::RenderPipeline,
    hdr_bind_group: wgpu::BindGroup,
    hdr_pipeline: wgpu::RenderPipeline,
    hdr_lod_pipeline: wgpu::RenderPipeline,
    #[cfg(feature = "assets")]
    cubemap_texture: wgpu::Texture,
    #[cfg(feature = "assets")]
    cubemap_view: wgpu::TextureView,
    #[cfg(feature = "assets")]
    sampler: wgpu::Sampler,
    hdr_loaded: bool,
    #[cfg(feature = "assets")]
    hdr_bind_group_layout: wgpu::BindGroupLayout,
    irradiance_bind_group: wgpu::BindGroup,
    prefilter_bind_group: wgpu::BindGroup,
    #[cfg(feature = "assets")]
    irradiance_texture: Option<wgpu::Texture>,
    #[cfg(feature = "assets")]
    prefilter_texture: Option<wgpu::Texture>,
    #[cfg(feature = "assets")]
    procedural_cubemap_pipeline: wgpu::ComputePipeline,
    #[cfg(feature = "assets")]
    procedural_cubemap_bind_group_layout: wgpu::BindGroupLayout,
    #[cfg(feature = "assets")]
    procedural_uniform_buffer: wgpu::Buffer,
    pub ibl_snapshots: Vec<(f32, wgpu::Texture, wgpu::Texture)>,
    pub ibl_snapshot_atmosphere: Option<crate::config::Atmosphere>,
}

/// Builds a full-screen sky render pipeline. All sky variants share the same
/// fixed-function state and `vs_sky` vertex stage, differing only in their
/// pipeline layout, shader module, and fragment entry point.
fn build_sky_pipeline(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    fragment_entry: &str,
    color_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_sky"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(fragment_entry),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

impl SkyPass {
    pub fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        _depth_format: wgpu::TextureFormat,
    ) -> Self {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sky Uniform Buffer"),
            size: std::mem::size_of::<SkyUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        #[cfg(feature = "assets")]
        let (cubemap_texture, cubemap_view) = create_dummy_cubemap(device);
        #[cfg(not(feature = "assets"))]
        let (_, cubemap_view) = create_dummy_cubemap(device);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let procedural_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Sky Procedural Bind Group Layout"),
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
            });

        let procedural_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &procedural_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Sky Procedural Bind Group"),
        });

        let hdr_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Sky HDR Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let hdr_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &hdr_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cubemap_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Sky HDR Bind Group"),
        });

        let sky_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "sky.wgsl",
            include_str!("../../shaders/sky.wgsl"),
        );
        let cloudy_sky_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "sky_cloudy.wgsl",
            include_str!("../../shaders/sky_cloudy.wgsl"),
        );
        let space_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "sky_space.wgsl",
            include_str!("../../shaders/sky_space.wgsl"),
        );
        let nebula_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "sky_nebula.wgsl",
            include_str!("../../shaders/sky_nebula.wgsl"),
        );
        let sunset_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "sky_sunset.wgsl",
            include_str!("../../shaders/sky_sunset.wgsl"),
        );
        let day_night_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "sky_day_night.wgsl",
            include_str!("../../shaders/sky_day_night.wgsl"),
        );
        let hdr_shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "sky_hdr.wgsl",
            include_str!("../../shaders/sky_hdr.wgsl"),
        );

        let procedural_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Sky Procedural Pipeline Layout"),
                bind_group_layouts: &[Some(&procedural_bind_group_layout)],
                immediate_size: 0,
            });

        let hdr_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sky HDR Pipeline Layout"),
            bind_group_layouts: &[Some(&hdr_bind_group_layout)],
            immediate_size: 0,
        });

        let sky_pipeline = build_sky_pipeline(
            device,
            "Sky Pipeline",
            &procedural_pipeline_layout,
            &sky_shader,
            "fs_sky",
            color_format,
        );

        let cloudy_sky_pipeline = build_sky_pipeline(
            device,
            "Cloudy Sky Pipeline",
            &procedural_pipeline_layout,
            &cloudy_sky_shader,
            "fs_sky",
            color_format,
        );

        let space_pipeline = build_sky_pipeline(
            device,
            "Space Pipeline",
            &procedural_pipeline_layout,
            &space_shader,
            "fs_sky",
            color_format,
        );

        let nebula_pipeline = build_sky_pipeline(
            device,
            "Nebula Pipeline",
            &procedural_pipeline_layout,
            &nebula_shader,
            "fs_sky",
            color_format,
        );

        let sunset_pipeline = build_sky_pipeline(
            device,
            "Sunset Pipeline",
            &procedural_pipeline_layout,
            &sunset_shader,
            "fs_sky",
            color_format,
        );
        let day_night_pipeline = build_sky_pipeline(
            device,
            "DayNight Pipeline",
            &procedural_pipeline_layout,
            &day_night_shader,
            "fs_sky",
            color_format,
        );

        let hdr_pipeline = build_sky_pipeline(
            device,
            "Sky HDR Pipeline",
            &hdr_pipeline_layout,
            &hdr_shader,
            "fs_sky",
            color_format,
        );

        let hdr_lod_pipeline = build_sky_pipeline(
            device,
            "Sky HDR LOD Pipeline",
            &hdr_pipeline_layout,
            &hdr_shader,
            "fs_sky_lod",
            color_format,
        );

        let irradiance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &hdr_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cubemap_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Sky Irradiance Bind Group"),
        });

        let prefilter_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &hdr_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cubemap_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Sky Prefilter Bind Group"),
        });

        #[cfg(feature = "assets")]
        let (
            procedural_uniform_buffer,
            procedural_cubemap_bind_group_layout,
            procedural_cubemap_pipeline,
        ) = {
            let procedural_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Procedural Cubemap Uniform Buffer"),
                size: 16,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let procedural_cubemap_bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Procedural to Cubemap Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: wgpu::TextureFormat::Rgba16Float,
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                            },
                            count: None,
                        },
                    ],
                });

            let procedural_cubemap_shader = crate::wgpu::shader_compose::compile_wgsl(
                device,
                "procedural_to_cubemap.wgsl",
                include_str!("../../shaders/procedural_to_cubemap.wgsl"),
            );

            let procedural_cubemap_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Procedural to Cubemap Pipeline Layout"),
                    bind_group_layouts: &[Some(&procedural_cubemap_bind_group_layout)],
                    immediate_size: 0,
                });

            let procedural_cubemap_pipeline =
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Procedural to Cubemap Pipeline"),
                    layout: Some(&procedural_cubemap_pipeline_layout),
                    module: &procedural_cubemap_shader,
                    entry_point: Some("main"),
                    compilation_options: Default::default(),
                    cache: None,
                });

            (
                procedural_uniform_buffer,
                procedural_cubemap_bind_group_layout,
                procedural_cubemap_pipeline,
            )
        };

        Self {
            uniform_buffer,
            procedural_bind_group,
            sky_pipeline,
            cloudy_sky_pipeline,
            space_pipeline,
            nebula_pipeline,
            sunset_pipeline,
            day_night_pipeline,
            hdr_bind_group,
            hdr_pipeline,
            hdr_lod_pipeline,
            #[cfg(feature = "assets")]
            cubemap_texture,
            #[cfg(feature = "assets")]
            cubemap_view,
            #[cfg(feature = "assets")]
            sampler,
            hdr_loaded: false,
            #[cfg(feature = "assets")]
            hdr_bind_group_layout,
            irradiance_bind_group,
            prefilter_bind_group,
            #[cfg(feature = "assets")]
            irradiance_texture: None,
            #[cfg(feature = "assets")]
            prefilter_texture: None,
            #[cfg(feature = "assets")]
            procedural_cubemap_pipeline,
            #[cfg(feature = "assets")]
            procedural_cubemap_bind_group_layout,
            #[cfg(feature = "assets")]
            procedural_uniform_buffer,
            ibl_snapshots: Vec::new(),
            ibl_snapshot_atmosphere: None,
        }
    }

    #[cfg(feature = "assets")]
    pub fn load_hdr_skybox(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
    ) -> (wgpu::TextureView, wgpu::TextureView) {
        let result = crate::wgpu::hdr::load_hdri_texture(device, queue, bytes);

        self.cubemap_texture = result.cubemap;
        self.cubemap_view = result.cubemap_view;
        self.hdr_loaded = true;

        let irradiance_view = result.filtered_maps.irradiance_view;
        let prefiltered_view = result.filtered_maps.prefiltered_view;

        self.irradiance_texture = Some(result.filtered_maps.irradiance_texture);
        self.prefilter_texture = Some(result.filtered_maps.prefiltered_texture);

        let irradiance_view_for_sky =
            self.irradiance_texture
                .as_ref()
                .unwrap()
                .create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::Cube),
                    ..Default::default()
                });

        let prefilter_view_for_sky =
            self.prefilter_texture
                .as_ref()
                .unwrap()
                .create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::Cube),
                    ..Default::default()
                });

        self.hdr_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.hdr_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.cubemap_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("Sky HDR Bind Group"),
        });

        self.irradiance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.hdr_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&irradiance_view_for_sky),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("Sky Irradiance Bind Group"),
        });

        self.prefilter_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.hdr_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&prefilter_view_for_sky),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("Sky Prefilter Bind Group"),
        });

        (irradiance_view, prefiltered_view)
    }

    #[cfg(feature = "assets")]
    pub fn capture_procedural_atmosphere(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atmosphere: Atmosphere,
        time: f32,
    ) -> Option<(wgpu::TextureView, wgpu::TextureView)> {
        let atmosphere_type = atmosphere.as_procedural_cubemap_type()?;

        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct ProceduralUniform {
            time: f32,
            atmosphere_type: u32,
            _padding0: u32,
            _padding1: u32,
        }

        let uniform = ProceduralUniform {
            time,
            atmosphere_type,
            _padding0: 0,
            _padding1: 0,
        };

        queue.write_buffer(
            &self.procedural_uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniform]),
        );

        let mip_level_count = calculate_mip_count(PROCEDURAL_CUBEMAP_SIZE);

        let cubemap = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Procedural Sky Cubemap"),
            size: wgpu::Extent3d {
                width: PROCEDURAL_CUBEMAP_SIZE,
                height: PROCEDURAL_CUBEMAP_SIZE,
                depth_or_array_layers: 6,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let cubemap_storage_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Procedural to Cubemap Bind Group"),
            layout: &self.procedural_cubemap_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.procedural_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&cubemap_storage_view),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Procedural to Cubemap Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Procedural to Cubemap Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.procedural_cubemap_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(64, 64, 6);
        }

        queue.submit(Some(encoder.finish()));

        generate_cubemap_mipmaps(device, queue, &cubemap);

        let cubemap_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let filtered_maps = filter_environment_map(device, queue, &cubemap, &cubemap_view);

        let irradiance_view = filtered_maps.irradiance_view;
        let prefiltered_view = filtered_maps.prefiltered_view;

        self.irradiance_texture = Some(filtered_maps.irradiance_texture);
        self.prefilter_texture = Some(filtered_maps.prefiltered_texture);

        let irradiance_view_for_sky =
            self.irradiance_texture
                .as_ref()
                .unwrap()
                .create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::Cube),
                    ..Default::default()
                });

        let prefilter_view_for_sky =
            self.prefilter_texture
                .as_ref()
                .unwrap()
                .create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::Cube),
                    ..Default::default()
                });

        self.irradiance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.hdr_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&irradiance_view_for_sky),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("Sky Irradiance Bind Group"),
        });

        self.prefilter_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.hdr_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&prefilter_view_for_sky),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("Sky Prefilter Bind Group"),
        });

        Some((irradiance_view, prefiltered_view))
    }

    #[cfg(feature = "assets")]
    pub fn capture_ibl_snapshots(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atmosphere: Atmosphere,
        hours: &[f32],
    ) {
        self.ibl_snapshots.clear();
        self.ibl_snapshot_atmosphere = Some(atmosphere);

        let atmosphere_type = match atmosphere.as_procedural_cubemap_type() {
            Some(t) => t,
            None => return,
        };

        for &hour in hours {
            #[repr(C)]
            #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
            struct ProceduralUniform {
                time: f32,
                atmosphere_type: u32,
                _padding0: u32,
                _padding1: u32,
            }

            let uniform = ProceduralUniform {
                time: hour,
                atmosphere_type,
                _padding0: 0,
                _padding1: 0,
            };

            queue.write_buffer(
                &self.procedural_uniform_buffer,
                0,
                bytemuck::cast_slice(&[uniform]),
            );

            let mip_level_count = calculate_mip_count(PROCEDURAL_CUBEMAP_SIZE);

            let cubemap = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("IBL Snapshot Cubemap"),
                size: wgpu::Extent3d {
                    width: PROCEDURAL_CUBEMAP_SIZE,
                    height: PROCEDURAL_CUBEMAP_SIZE,
                    depth_or_array_layers: 6,
                },
                mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            });

            let cubemap_storage_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                base_mip_level: 0,
                mip_level_count: Some(1),
                ..Default::default()
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("IBL Snapshot Cubemap Bind Group"),
                layout: &self.procedural_cubemap_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.procedural_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&cubemap_storage_view),
                    },
                ],
            });

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("IBL Snapshot Encoder"),
            });

            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("IBL Snapshot Compute Pass"),
                    timestamp_writes: None,
                });

                compute_pass.set_pipeline(&self.procedural_cubemap_pipeline);
                compute_pass.set_bind_group(0, &bind_group, &[]);
                compute_pass.dispatch_workgroups(64, 64, 6);
            }

            queue.submit(Some(encoder.finish()));

            generate_cubemap_mipmaps(device, queue, &cubemap);

            let cubemap_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::Cube),
                ..Default::default()
            });

            let filtered_maps = filter_environment_map(device, queue, &cubemap, &cubemap_view);

            self.ibl_snapshots.push((
                hour,
                filtered_maps.irradiance_texture,
                filtered_maps.prefiltered_texture,
            ));
        }
    }

    pub fn select_ibl_bracket(
        &self,
        atmosphere: Atmosphere,
        hour: f32,
    ) -> Option<(usize, usize, f32)> {
        if self.ibl_snapshots.len() < 2 {
            return None;
        }

        if self.ibl_snapshot_atmosphere != Some(atmosphere) {
            return None;
        }

        let hour = hour - (hour / 24.0).floor() * 24.0;

        let count = self.ibl_snapshots.len();

        let mut lower_index = count - 1;
        let mut upper_index = 0;

        for index in 0..count {
            if self.ibl_snapshots[index].0 <= hour {
                lower_index = index;
            }
        }

        for index in 0..count {
            if self.ibl_snapshots[index].0 > hour {
                upper_index = index;
                break;
            } else if index == count - 1 {
                upper_index = 0;
            }
        }

        let lower_hour = self.ibl_snapshots[lower_index].0;
        let upper_hour = self.ibl_snapshots[upper_index].0;

        let span = if upper_hour > lower_hour {
            upper_hour - lower_hour
        } else {
            (24.0 - lower_hour) + upper_hour
        };

        let elapsed = if hour >= lower_hour {
            hour - lower_hour
        } else {
            (24.0 - lower_hour) + hour
        };

        let blend = if span > 0.0 { elapsed / span } else { 0.0 };

        Some((lower_index, upper_index, blend.clamp(0.0, 1.0)))
    }
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for SkyPass {
    fn name(&self) -> &str {
        "sky_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec![]
    }

    fn writes(&self) -> Vec<&str> {
        vec![]
    }

    fn reads_writes(&self) -> Vec<&str> {
        vec!["color"]
    }

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        if let Some(camera_matrices) = configs.scene.render_view.as_ref() {
            let atmosphere = configs.scene.active_view.atmosphere;
            let uniform = SkyUniform::new(
                camera_matrices.projection,
                camera_matrices.view,
                camera_matrices.camera_position,
                configs.view.uptime_milliseconds as f32 / 1000.0,
                atmosphere.mip_level(),
                configs.scene.day_night.hour,
            );

            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
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

        let atmosphere = context.configs.scene.active_view.atmosphere;
        let draw_sky = atmosphere != Atmosphere::None && context.configs.settings.show_sky;

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;

        let color_load = match color_load {
            wgpu::LoadOp::Clear(_) => {
                let clear_color = if let Some(ref fog) = context.configs.scene.active_view.fog {
                    wgpu::Color {
                        r: fog.color[0] as f64,
                        g: fog.color[1] as f64,
                        b: fog.color[2] as f64,
                        a: 1.0,
                    }
                } else {
                    let cc = context.configs.settings.clear_color;
                    wgpu::Color {
                        r: cc[0] as f64,
                        g: cc[1] as f64,
                        b: cc[2] as f64,
                        a: cc[3] as f64,
                    }
                };
                wgpu::LoadOp::Clear(clear_color)
            }
            other => other,
        };

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Sky Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: color_load,
                        store: color_store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        if !draw_sky {
            drop(render_pass);
            return Ok(context.into_sub_graph_commands());
        }

        match atmosphere {
            Atmosphere::None => unreachable!(),
            Atmosphere::Hdr if self.hdr_loaded => {
                render_pass.set_pipeline(&self.hdr_pipeline);
                render_pass.set_bind_group(0, &self.hdr_bind_group, &[]);
            }
            Atmosphere::Sky | Atmosphere::Hdr => {
                render_pass.set_pipeline(&self.sky_pipeline);
                render_pass.set_bind_group(0, &self.procedural_bind_group, &[]);
            }
            Atmosphere::Space => {
                render_pass.set_pipeline(&self.space_pipeline);
                render_pass.set_bind_group(0, &self.procedural_bind_group, &[]);
            }
            Atmosphere::Nebula => {
                render_pass.set_pipeline(&self.nebula_pipeline);
                render_pass.set_bind_group(0, &self.procedural_bind_group, &[]);
            }
            Atmosphere::Sunset => {
                render_pass.set_pipeline(&self.sunset_pipeline);
                render_pass.set_bind_group(0, &self.procedural_bind_group, &[]);
            }
            Atmosphere::DayNight => {
                render_pass.set_pipeline(&self.day_night_pipeline);
                render_pass.set_bind_group(0, &self.procedural_bind_group, &[]);
            }
            Atmosphere::CloudySky => {
                render_pass.set_pipeline(&self.cloudy_sky_pipeline);
                render_pass.set_bind_group(0, &self.procedural_bind_group, &[]);
            }
            Atmosphere::Irradiance if self.hdr_loaded => {
                render_pass.set_pipeline(&self.hdr_pipeline);
                render_pass.set_bind_group(0, &self.irradiance_bind_group, &[]);
            }
            Atmosphere::PrefilterMip0
            | Atmosphere::PrefilterMip1
            | Atmosphere::PrefilterMip2
            | Atmosphere::PrefilterMip3
            | Atmosphere::PrefilterMip4
                if self.hdr_loaded =>
            {
                render_pass.set_pipeline(&self.hdr_lod_pipeline);
                render_pass.set_bind_group(0, &self.prefilter_bind_group, &[]);
            }
            Atmosphere::HdrMip0
            | Atmosphere::HdrMip1
            | Atmosphere::HdrMip2
            | Atmosphere::HdrMip3
            | Atmosphere::HdrMip4
                if self.hdr_loaded =>
            {
                render_pass.set_pipeline(&self.hdr_lod_pipeline);
                render_pass.set_bind_group(0, &self.hdr_bind_group, &[]);
            }
            Atmosphere::Irradiance
            | Atmosphere::PrefilterMip0
            | Atmosphere::PrefilterMip1
            | Atmosphere::PrefilterMip2
            | Atmosphere::PrefilterMip3
            | Atmosphere::PrefilterMip4
            | Atmosphere::HdrMip0
            | Atmosphere::HdrMip1
            | Atmosphere::HdrMip2
            | Atmosphere::HdrMip3
            | Atmosphere::HdrMip4 => {
                render_pass.set_pipeline(&self.sky_pipeline);
                render_pass.set_bind_group(0, &self.procedural_bind_group, &[]);
            }
        }
        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkyUniform {
    proj: [[f32; 4]; 4],
    proj_inv: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    cam_pos: [f32; 4],
    time: f32,
    mip_level: f32,
    hour: f32,
    _padding: f32,
}

impl SkyUniform {
    pub fn new(
        projection: nalgebra_glm::Mat4,
        view: nalgebra_glm::Mat4,
        camera_position: nalgebra_glm::Vec3,
        time: f32,
        mip_level: f32,
        hour: f32,
    ) -> Self {
        let proj: [[f32; 4]; 4] = projection.into();
        let proj_inv: [[f32; 4]; 4] = nalgebra_glm::inverse(&projection).into();
        let view: [[f32; 4]; 4] = view.into();
        let cam_pos = [camera_position.x, camera_position.y, camera_position.z, 1.0];

        Self {
            proj,
            proj_inv,
            view,
            cam_pos,
            time,
            mip_level,
            hour,
            _padding: 0.0,
        }
    }
}
