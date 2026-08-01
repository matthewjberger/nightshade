use crate::systems::envmap_filter::filter_environment_map;
use crate::systems::hdr::generate_cubemap_mipmaps;
use nightshade_render::data::config::Atmosphere;
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, PassPosition};
use nightshade_wgpu::wgpu;

const PROCEDURAL_CUBEMAP_SIZE: u32 = 1024;

fn calculate_mip_count(size: u32) -> u32 {
    (size as f32).log2().floor() as u32 + 1
}

fn create_dummy_cubemap(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> (wgpu::Texture, wgpu::TextureView) {
    let cubemap = nightshade_wgpu::backend::placeholder::placeholder_texture(
        device,
        queue,
        "Dummy Cubemap Texture",
        wgpu::TextureFormat::Rgba16Float,
        6,
        &nightshade_wgpu::backend::placeholder::TRANSPARENT_RGBA16,
    );

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
    cubemap_texture: wgpu::Texture,
    cubemap_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    hdr_loaded: bool,
    hdr_bind_group_layout: wgpu::BindGroupLayout,
    irradiance_bind_group: wgpu::BindGroup,
    prefilter_bind_group: wgpu::BindGroup,
    irradiance_texture: Option<wgpu::Texture>,
    prefilter_texture: Option<wgpu::Texture>,
    procedural_cubemap_pipeline: wgpu::ComputePipeline,
    procedural_cubemap_bind_group_layout: wgpu::BindGroupLayout,
    procedural_uniform_buffer: wgpu::Buffer,
    pub ibl_snapshots: Vec<(f32, wgpu::Texture, wgpu::Texture)>,
    pub ibl_snapshot_atmosphere: Option<nightshade_render::data::config::Atmosphere>,
    baked_texture: Option<wgpu::Texture>,
    baked_bind_group: Option<wgpu::BindGroup>,
    prefer_baked_sky: bool,
}

fn build_sky_pipeline(
    device: &wgpu::Device,
    label: &str,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    fragment_entry: &str,
    color_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    build::Pipe::new(device, label, shader)
        .layout(layout)
        .target(wgpu::ColorTargetState {
            format: color_format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        })
        .vertex("vs_sky")
        .fragment(fragment_entry)
        .make()
}

impl SkyPass {
    pub fn new(
        build: PassBuild,
        color_format: wgpu::TextureFormat,
        _depth_format: wgpu::TextureFormat,
        prefer_baked_sky: bool,
    ) -> Self {
        let PassBuild {
            device,
            queue,
            composer,
            ..
        } = build;
        let uniform_buffer = build::uniform(
            device,
            "Sky Uniform Buffer",
            std::mem::size_of::<SkyUniform>() as u64,
        );

        let (cubemap_texture, cubemap_view) = create_dummy_cubemap(device, queue);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let procedural_bind_group_layout = build::bind_layout(
            device,
            "Sky Procedural Bind Group Layout",
            &[build::uniform_at(0, wgpu::ShaderStages::VERTEX_FRAGMENT)],
        );

        let procedural_bind_group = build::binding(
            device,
            "Sky Procedural Bind Group",
            &procedural_bind_group_layout,
            vec![uniform_buffer.as_entire_binding()],
        );

        let hdr_bind_group_layout = build::bind_layout(
            device,
            "Sky HDR Bind Group Layout",
            &[
                build::uniform_at(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
                build::texture_at(
                    1,
                    wgpu::TextureSampleType::Float { filterable: true },
                    wgpu::TextureViewDimension::Cube,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::filtering_at(2),
            ],
        );

        let hdr_bind_group = build::binding(
            device,
            "Sky HDR Bind Group",
            &hdr_bind_group_layout,
            vec![
                uniform_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&cubemap_view),
                wgpu::BindingResource::Sampler(&sampler),
            ],
        );

        let sky_shader =
            composer.compile_wgsl(device, "sky.wgsl", include_str!("../shaders/sky.wgsl"));
        let cloudy_sky_shader = composer.compile_wgsl(
            device,
            "sky_cloudy.wgsl",
            include_str!("../shaders/sky_cloudy.wgsl"),
        );
        let space_shader = composer.compile_wgsl(
            device,
            "sky_space.wgsl",
            include_str!("../shaders/sky_space.wgsl"),
        );
        let nebula_shader = composer.compile_wgsl(
            device,
            "sky_nebula.wgsl",
            include_str!("../shaders/sky_nebula.wgsl"),
        );
        let sunset_shader = composer.compile_wgsl(
            device,
            "sky_sunset.wgsl",
            include_str!("../shaders/sky_sunset.wgsl"),
        );
        let day_night_shader = composer.compile_wgsl(
            device,
            "sky_day_night.wgsl",
            include_str!("../shaders/sky_day_night.wgsl"),
        );
        let hdr_shader = composer.compile_wgsl(
            device,
            "sky_hdr.wgsl",
            include_str!("../shaders/sky_hdr.wgsl"),
        );

        let procedural_pipeline_layout = build::pipeline_layout(
            device,
            "Sky Procedural Pipeline Layout",
            &[&procedural_bind_group_layout],
        );

        let hdr_pipeline_layout =
            build::pipeline_layout(device, "Sky HDR Pipeline Layout", &[&hdr_bind_group_layout]);

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

        let irradiance_bind_group = build::binding(
            device,
            "Sky Irradiance Bind Group",
            &hdr_bind_group_layout,
            vec![
                uniform_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&cubemap_view),
                wgpu::BindingResource::Sampler(&sampler),
            ],
        );

        let prefilter_bind_group = build::binding(
            device,
            "Sky Prefilter Bind Group",
            &hdr_bind_group_layout,
            vec![
                uniform_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&cubemap_view),
                wgpu::BindingResource::Sampler(&sampler),
            ],
        );

        let (
            procedural_uniform_buffer,
            procedural_cubemap_bind_group_layout,
            procedural_cubemap_pipeline,
        ) = {
            let procedural_uniform_buffer =
                build::uniform(device, "Procedural Cubemap Uniform Buffer", 16);

            let procedural_cubemap_bind_group_layout = build::bind_layout(
                device,
                "Procedural to Cubemap Bind Group Layout",
                &[
                    build::uniform_at(0, wgpu::ShaderStages::COMPUTE),
                    build::storage_texture_at(
                        1,
                        wgpu::TextureFormat::Rgba16Float,
                        wgpu::StorageTextureAccess::WriteOnly,
                        wgpu::TextureViewDimension::D2Array,
                        wgpu::ShaderStages::COMPUTE,
                    ),
                ],
            );

            let procedural_cubemap_shader = composer.compile_wgsl(
                device,
                "procedural_to_cubemap.wgsl",
                include_str!("../shaders/procedural_to_cubemap.wgsl"),
            );

            let procedural_cubemap_pipeline_layout = build::pipeline_layout(
                device,
                "Procedural to Cubemap Pipeline Layout",
                &[&procedural_cubemap_bind_group_layout],
            );

            let procedural_cubemap_pipeline = build::compute(
                device,
                "Procedural to Cubemap Pipeline",
                &procedural_cubemap_shader,
                "main",
                &procedural_cubemap_pipeline_layout,
            );

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
            cubemap_texture,
            cubemap_view,
            sampler,
            hdr_loaded: false,
            hdr_bind_group_layout,
            irradiance_bind_group,
            prefilter_bind_group,
            irradiance_texture: None,
            prefilter_texture: None,
            procedural_cubemap_pipeline,
            procedural_cubemap_bind_group_layout,
            procedural_uniform_buffer,
            ibl_snapshots: Vec::new(),
            ibl_snapshot_atmosphere: None,
            baked_texture: None,
            baked_bind_group: None,
            prefer_baked_sky,
        }
    }

    pub fn load_hdr_skybox(
        &mut self,
        build: PassBuild,
        queue: &wgpu::Queue,
        bytes: &[u8],
    ) -> (wgpu::TextureView, wgpu::TextureView) {
        let PassBuild { device, .. } = build;
        let result = crate::systems::hdr::load_hdri_texture(build, queue, bytes);

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

        self.hdr_bind_group = build::binding(
            device,
            "Sky HDR Bind Group",
            &self.hdr_bind_group_layout,
            vec![
                self.uniform_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&self.cubemap_view),
                wgpu::BindingResource::Sampler(&self.sampler),
            ],
        );

        self.irradiance_bind_group = build::binding(
            device,
            "Sky Irradiance Bind Group",
            &self.hdr_bind_group_layout,
            vec![
                self.uniform_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&irradiance_view_for_sky),
                wgpu::BindingResource::Sampler(&self.sampler),
            ],
        );

        self.prefilter_bind_group = build::binding(
            device,
            "Sky Prefilter Bind Group",
            &self.hdr_bind_group_layout,
            vec![
                self.uniform_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&prefilter_view_for_sky),
                wgpu::BindingResource::Sampler(&self.sampler),
            ],
        );

        (irradiance_view, prefiltered_view)
    }

    pub fn capture_procedural_atmosphere(
        &mut self,
        build: PassBuild,
        queue: &wgpu::Queue,
        atmosphere: Atmosphere,
        time: f32,
    ) -> Option<(wgpu::TextureView, wgpu::TextureView)> {
        let PassBuild { device, .. } = build;
        let atmosphere_type = atmosphere.as_procedural_cubemap_type()?;

        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct ProceduralUniform {
            time: f32,
            atmosphere_type: u32,
            padding_0: u32,
            padding_1: u32,
        }

        let uniform = ProceduralUniform {
            time,
            atmosphere_type,
            padding_0: 0,
            padding_1: 0,
        };

        queue.write_buffer(
            &self.procedural_uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniform]),
        );

        let mip_level_count = calculate_mip_count(PROCEDURAL_CUBEMAP_SIZE);

        let cubemap = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Procedural Sky Cubemap"),
            size: nightshade_wgpu::backend::texture_extent(
                device,
                PROCEDURAL_CUBEMAP_SIZE,
                PROCEDURAL_CUBEMAP_SIZE,
                6,
            ),
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

        let bind_group = build::binding(
            device,
            "Procedural to Cubemap Bind Group",
            &self.procedural_cubemap_bind_group_layout,
            vec![
                self.procedural_uniform_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&cubemap_storage_view),
            ],
        );

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

        generate_cubemap_mipmaps(build, queue, &cubemap);

        let cubemap_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        self.baked_bind_group = Some(build::binding(
            device,
            "Sky Baked Bind Group",
            &self.hdr_bind_group_layout,
            vec![
                self.uniform_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&cubemap_view),
                wgpu::BindingResource::Sampler(&self.sampler),
            ],
        ));

        let filtered_maps = filter_environment_map(build, queue, &cubemap, &cubemap_view);
        self.baked_texture = Some(cubemap);

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

        self.irradiance_bind_group = build::binding(
            device,
            "Sky Irradiance Bind Group",
            &self.hdr_bind_group_layout,
            vec![
                self.uniform_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&irradiance_view_for_sky),
                wgpu::BindingResource::Sampler(&self.sampler),
            ],
        );

        self.prefilter_bind_group = build::binding(
            device,
            "Sky Prefilter Bind Group",
            &self.hdr_bind_group_layout,
            vec![
                self.uniform_buffer.as_entire_binding(),
                wgpu::BindingResource::TextureView(&prefilter_view_for_sky),
                wgpu::BindingResource::Sampler(&self.sampler),
            ],
        );

        Some((irradiance_view, prefiltered_view))
    }

    pub fn capture_ibl_snapshots(
        &mut self,
        build: PassBuild,
        queue: &wgpu::Queue,
        atmosphere: Atmosphere,
        hours: &[f32],
    ) {
        let PassBuild { device, .. } = build;
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
                padding_0: u32,
                padding_1: u32,
            }

            let uniform = ProceduralUniform {
                time: hour,
                atmosphere_type,
                padding_0: 0,
                padding_1: 0,
            };

            queue.write_buffer(
                &self.procedural_uniform_buffer,
                0,
                bytemuck::cast_slice(&[uniform]),
            );

            let mip_level_count = calculate_mip_count(PROCEDURAL_CUBEMAP_SIZE);

            let cubemap = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("IBL Snapshot Cubemap"),
                size: nightshade_wgpu::backend::texture_extent(
                    device,
                    PROCEDURAL_CUBEMAP_SIZE,
                    PROCEDURAL_CUBEMAP_SIZE,
                    6,
                ),
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

            let bind_group = build::binding(
                device,
                "IBL Snapshot Cubemap Bind Group",
                &self.procedural_cubemap_bind_group_layout,
                vec![
                    self.procedural_uniform_buffer.as_entire_binding(),
                    wgpu::BindingResource::TextureView(&cubemap_storage_view),
                ],
            );

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

            generate_cubemap_mipmaps(build, queue, &cubemap);

            let cubemap_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::Cube),
                ..Default::default()
            });

            let filtered_maps = filter_environment_map(build, queue, &cubemap, &cubemap_view);

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

const SKY_PASS_DECL: PassDecl = PassDecl {
    name: "sky_pass",
    position: PassPosition::Background,
    reads_writes: &["color"],
    ..PassDecl::EMPTY
};

impl PassNode<nightshade_wgpu::backend::RenderInputs> for SkyPass {
    fn decl(&self) -> &PassDecl {
        &SKY_PASS_DECL
    }

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
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
        context: PassExecutionContext<'r, 'e, nightshade_wgpu::backend::RenderInputs>,
    ) -> nightshade_wgpu::backend::Result<()> {
        if !context.is_pass_enabled() {
            return Ok(());
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
            return Ok(());
        }

        let read_the_bake = self.prefer_baked_sky
            && atmosphere.is_procedural()
            && atmosphere != Atmosphere::DayNight;
        if read_the_bake && let Some(bind_group) = self.baked_bind_group.as_ref() {
            render_pass.set_pipeline(&self.hdr_pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.draw(0..3, 0..1);
            drop(render_pass);
            return Ok(());
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

        Ok(())
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
    padding: f32,
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
            padding: 0.0,
        }
    }
}
