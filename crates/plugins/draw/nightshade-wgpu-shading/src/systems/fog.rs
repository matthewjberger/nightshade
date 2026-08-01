use nightshade_render::data::config::Fog;
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

pub const FOG_X: u32 = 160;
pub const FOG_Y: u32 = 90;
pub const FOG_Z: u32 = 64;

const FRAME_WRAP: u32 = 4096;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct FogUniforms {
    air: [f32; 4],
    screen: [f32; 2],
    focal: [f32; 2],
    far: f32,
    frame: u32,
    volumetric: u32,
    resolving: u32,
}

#[derive(Clone)]
pub struct AirReads {
    pub volume: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub uniforms: wgpu::Buffer,
}

pub struct ScatterReads<'a> {
    pub shadow_depth: &'a wgpu::TextureView,
    pub spotlight_shadow_atlas: &'a wgpu::TextureView,
    pub compare_sampler: &'a wgpu::Sampler,
    pub spotlight_shadows: &'a wgpu::Buffer,
    pub point_shadow_cubemap: &'a wgpu::TextureView,
    pub point_shadow_sampler: &'a wgpu::Sampler,
    pub point_light_shadows: &'a wgpu::Buffer,
    pub irradiance: &'a wgpu::TextureView,
    pub irradiance_b: &'a wgpu::TextureView,
    pub ibl_sampler: &'a wgpu::Sampler,
}

pub struct Air {
    scatter: [wgpu::TextureView; 2],
    marched: wgpu::TextureView,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    scatter_layout: wgpu::BindGroupLayout,
    scatter_pipeline: wgpu::ComputePipeline,
    scatter_groups: Option<[wgpu::BindGroup; 2]>,
    march_pipeline: wgpu::ComputePipeline,
    march_groups: [wgpu::BindGroup; 2],
    sky_pipeline: wgpu::RenderPipeline,
    sky_group: wgpu::BindGroup,
    frame: u32,
}

fn create_grid(device: &wgpu::Device, label: &str) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: nightshade_wgpu::backend::texture_extent(device, FOG_X, FOG_Y, FOG_Z),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

fn compute_entry(binding: u32, ty: wgpu::BindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty,
        count: None,
    }
}

const UNIFORM: wgpu::BindingType = wgpu::BindingType::Buffer {
    ty: wgpu::BufferBindingType::Uniform,
    has_dynamic_offset: false,
    min_binding_size: None,
};

const STORAGE_READ: wgpu::BindingType = wgpu::BindingType::Buffer {
    ty: wgpu::BufferBindingType::Storage { read_only: true },
    has_dynamic_offset: false,
    min_binding_size: None,
};

const GRID_WRITE: wgpu::BindingType = wgpu::BindingType::StorageTexture {
    access: wgpu::StorageTextureAccess::WriteOnly,
    format: wgpu::TextureFormat::Rgba16Float,
    view_dimension: wgpu::TextureViewDimension::D3,
};

const GRID_READ: wgpu::BindingType = wgpu::BindingType::Texture {
    sample_type: wgpu::TextureSampleType::Float { filterable: true },
    view_dimension: wgpu::TextureViewDimension::D3,
    multisampled: false,
};

const DEPTH_READ: wgpu::BindingType = wgpu::BindingType::Texture {
    sample_type: wgpu::TextureSampleType::Depth,
    view_dimension: wgpu::TextureViewDimension::D2,
    multisampled: false,
};

const CUBE_READ: wgpu::BindingType = wgpu::BindingType::Texture {
    sample_type: wgpu::TextureSampleType::Float { filterable: true },
    view_dimension: wgpu::TextureViewDimension::Cube,
    multisampled: false,
};

impl Air {
    pub fn new(
        build: PassBuild,
        uniform_layout: &wgpu::BindGroupLayout,
        instance_layout: &wgpu::BindGroupLayout,
        texture_layout: &wgpu::BindGroupLayout,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        material_shader_defs: &[(&str, naga_oil::compose::ShaderDefValue)],
    ) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let scatter = [
            create_grid(device, "Fog Scatter Grid A"),
            create_grid(device, "Fog Scatter Grid B"),
        ];
        let marched = create_grid(device, "Fog Marched Grid");
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Fog Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let uniforms = build::uniform(
            device,
            "Fog Uniform Buffer",
            std::mem::size_of::<FogUniforms>() as u64,
        );

        let scatter_layout = build::bind_layout(
            device,
            "Fog Scatter Bind Group Layout",
            &[
                compute_entry(0, DEPTH_READ),
                compute_entry(
                    1,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                ),
                compute_entry(2, DEPTH_READ),
                compute_entry(3, STORAGE_READ),
                compute_entry(
                    4,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::CubeArray,
                        multisampled: false,
                    },
                ),
                compute_entry(
                    5,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                ),
                compute_entry(6, STORAGE_READ),
                compute_entry(7, CUBE_READ),
                compute_entry(8, CUBE_READ),
                compute_entry(
                    9,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
                compute_entry(10, UNIFORM),
                compute_entry(11, GRID_WRITE),
                compute_entry(12, GRID_READ),
                compute_entry(
                    13,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
            ],
        );
        let scatter_shader = composer.compile_wgsl_with_defs(
            device,
            "fog_scatter.wgsl",
            include_str!("../shaders/fog_scatter.wgsl"),
            material_shader_defs,
        );
        let scatter_pipeline_layout = build::pipeline_layout(
            device,
            "Fog Scatter Pipeline Layout",
            &[
                uniform_layout,
                instance_layout,
                texture_layout,
                &scatter_layout,
            ],
        );
        let scatter_pipeline = build::compute(
            device,
            "Fog Scatter Pipeline",
            &scatter_shader,
            "main",
            &scatter_pipeline_layout,
        );

        let march_layout = build::bind_layout(
            device,
            "Fog March Bind Group Layout",
            &[
                compute_entry(0, UNIFORM),
                compute_entry(1, GRID_READ),
                compute_entry(2, GRID_WRITE),
            ],
        );
        let march_shader = composer.compile_wgsl(
            device,
            "fog_march.wgsl",
            include_str!("../shaders/fog_march.wgsl"),
        );
        let march_pipeline_layout =
            build::pipeline_layout(device, "Fog March Pipeline Layout", &[&march_layout]);
        let march_pipeline = build::compute(
            device,
            "Fog March Pipeline",
            &march_shader,
            "main",
            &march_pipeline_layout,
        );
        let march_groups = [0, 1].map(|parity| {
            build::binding(
                device,
                "Fog March Bind Group",
                &march_layout,
                vec![
                    uniforms.as_entire_binding(),
                    wgpu::BindingResource::TextureView(&scatter[parity]),
                    wgpu::BindingResource::TextureView(&marched),
                ],
            )
        });

        let sky_layout = build::bind_layout(
            device,
            "Fog Sky Bind Group Layout",
            &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: UNIFORM,
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: GRID_READ,
                    count: None,
                },
                build::filtering_at(2),
            ],
        );
        let sky_group = build::binding(
            device,
            "Fog Sky Bind Group",
            &sky_layout,
            vec![
                uniforms.as_entire_binding(),
                wgpu::BindingResource::TextureView(&marched),
                wgpu::BindingResource::Sampler(&sampler),
            ],
        );
        let sky_shader = composer.compile_wgsl(
            device,
            "fog_sky.wgsl",
            include_str!("../shaders/fog_sky.wgsl"),
        );
        let sky_pipeline_layout =
            build::pipeline_layout(device, "Fog Sky Pipeline Layout", &[&sky_layout]);
        let sky_pipeline = build::Pipe::new(device, "Fog Sky Pipeline", &sky_shader)
            .layout(&sky_pipeline_layout)
            .target(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::SrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })
            .depth(build::depth_matched(depth_format))
            .vertex("vs_main")
            .fragment("fs_main")
            .make();

        Self {
            scatter,
            marched,
            sampler,
            uniforms,
            scatter_layout,
            scatter_pipeline,
            scatter_groups: None,
            march_pipeline,
            march_groups,
            sky_pipeline,
            sky_group,
            frame: FRAME_WRAP - 1,
        }
    }

    pub fn reads(&self) -> AirReads {
        AirReads {
            volume: self.marched.clone(),
            sampler: self.sampler.clone(),
            uniforms: self.uniforms.clone(),
        }
    }

    pub fn has_groups(&self) -> bool {
        self.scatter_groups.is_some()
    }

    pub fn rebuild_scatter_groups(&mut self, device: &wgpu::Device, reads: ScatterReads) {
        let ScatterReads {
            shadow_depth,
            spotlight_shadow_atlas,
            compare_sampler,
            spotlight_shadows,
            point_shadow_cubemap,
            point_shadow_sampler,
            point_light_shadows,
            irradiance,
            irradiance_b,
            ibl_sampler,
        } = reads;
        self.scatter_groups = Some([0, 1].map(|parity| {
            build::binding(
                device,
                "Fog Scatter Bind Group",
                &self.scatter_layout,
                vec![
                    wgpu::BindingResource::TextureView(shadow_depth),
                    wgpu::BindingResource::Sampler(compare_sampler),
                    wgpu::BindingResource::TextureView(spotlight_shadow_atlas),
                    spotlight_shadows.as_entire_binding(),
                    wgpu::BindingResource::TextureView(point_shadow_cubemap),
                    wgpu::BindingResource::Sampler(point_shadow_sampler),
                    point_light_shadows.as_entire_binding(),
                    wgpu::BindingResource::TextureView(irradiance),
                    wgpu::BindingResource::TextureView(irradiance_b),
                    wgpu::BindingResource::Sampler(ibl_sampler),
                    self.uniforms.as_entire_binding(),
                    wgpu::BindingResource::TextureView(&self.scatter[parity]),
                    wgpu::BindingResource::TextureView(&self.scatter[1 - parity]),
                    wgpu::BindingResource::Sampler(&self.sampler),
                ],
            )
        }));
    }

    pub fn write_uniforms(
        &mut self,
        queue: &wgpu::Queue,
        fog: Option<&Fog>,
        screen: (u32, u32),
        focal: [f32; 2],
        resolving: bool,
    ) {
        self.frame = (self.frame + 1) % FRAME_WRAP;
        let uniforms = FogUniforms {
            air: fog
                .map(|fog| [fog.density, fog.anisotropy, fog.height_falloff, fog.height])
                .unwrap_or([0.0; 4]),
            screen: [screen.0 as f32, screen.1 as f32],
            focal,
            far: fog.map(|fog| fog.end).unwrap_or(1.0),
            frame: self.frame,
            volumetric: u32::from(fog.is_some()),
            resolving: u32::from(resolving),
        };
        queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&uniforms));
    }

    pub fn light(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        uniforms: &wgpu::BindGroup,
        instances: &wgpu::BindGroup,
        materials: &wgpu::BindGroup,
    ) {
        let Some(scatter_groups) = self.scatter_groups.as_ref() else {
            return;
        };
        let parity = (self.frame & 1) as usize;
        let mut scatter = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Fog Scatter Pass"),
            timestamp_writes: None,
        });
        scatter.set_pipeline(&self.scatter_pipeline);
        scatter.set_bind_group(0, uniforms, &[]);
        scatter.set_bind_group(1, instances, &[]);
        scatter.set_bind_group(2, materials, &[]);
        scatter.set_bind_group(3, &scatter_groups[parity], &[]);
        scatter.dispatch_workgroups(FOG_X.div_ceil(8), FOG_Y.div_ceil(8), FOG_Z.div_ceil(4));
        drop(scatter);

        let mut march = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Fog March Pass"),
            timestamp_writes: None,
        });
        march.set_pipeline(&self.march_pipeline);
        march.set_bind_group(0, &self.march_groups[parity], &[]);
        march.dispatch_workgroups(FOG_X.div_ceil(8), FOG_Y.div_ceil(8), 1);
        drop(march);
    }

    pub fn fog_the_sky(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        depth_load: wgpu::LoadOp<f32>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Fog Sky Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: depth_load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.sky_pipeline);
        pass.set_bind_group(0, &self.sky_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
