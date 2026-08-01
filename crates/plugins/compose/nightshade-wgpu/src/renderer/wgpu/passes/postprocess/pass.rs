use crate::renderer::wgpu::build;
use crate::renderer::wgpu::pass_build::PassBuild;
use crate::renderer::wgpu::rendergraph::{PassDecl, PassExecutionContext, PassNode, PassPosition};
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
    time: f32,
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

pub struct PostProcessPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    cached_bind_group: Option<wgpu::BindGroup>,
    hdr_source_is_dof_output: bool,
    letterbox_amount: f32,
    fallback_texture_view: wgpu::TextureView,
    auto_exposure_pipeline: wgpu::ComputePipeline,
    auto_exposure_bind_group_layout: wgpu::BindGroupLayout,
    auto_exposure_bind_group: Option<wgpu::BindGroup>,
    auto_exposure_gather_pipeline: wgpu::ComputePipeline,
    auto_exposure_buffer: wgpu::Buffer,
    color_lut_texture: wgpu::Texture,
    color_lut_view: wgpu::TextureView,
    color_lut_sampler: wgpu::Sampler,
    pending_color_lut: Option<Vec<u8>>,
    output_encodes_srgb: bool,
}

const COLOR_LUT_SIZE: u32 = 16;

const EXPOSURE_STEPS: usize = 256;
const EXPOSURE_HEAD: usize = 8;
const EXPOSURE_GATHER_SPAN: u32 = 32;

const DARKEST_READING: f32 = -10.0;
const BRIGHTEST_READING: f32 = 4.0;

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
    pub fn new(
        build: PassBuild,
        surface_format: wgpu::TextureFormat,
        bloom_intensity: f32,
    ) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let color_lut_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Color Grading LUT"),
            size: crate::renderer::texture_extent::texture_extent(
                device,
                COLOR_LUT_SIZE,
                COLOR_LUT_SIZE,
                COLOR_LUT_SIZE,
            ),
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
        let shader = composer.compile_wgsl(
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
            time: 0.0,
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

        let fallback_texture = build::surface(
            device,
            "PostProcess Fallback Texture",
            wgpu::TextureFormat::Rgba16Float,
            [1, 1, 1],
            1,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );

        let fallback_texture_view =
            fallback_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group_layout = build::bind_layout(
            device,
            "PostProcess Bind Group Layout",
            &[
                build::painted_at(0),
                build::filtering_at(1),
                build::uniform_at(2, wgpu::ShaderStages::FRAGMENT),
                build::painted_at(3),
                build::filtering_at(4),
                build::painted_at(5),
                build::filtering_at(6),
                build::painted_at(7),
                build::filtering_at(8),
                build::painted_at(9),
                build::filtering_at(10),
                build::storage_at(11, false, wgpu::ShaderStages::FRAGMENT),
                build::texture_at(
                    12,
                    wgpu::TextureSampleType::Float { filterable: true },
                    wgpu::TextureViewDimension::D3,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::filtering_at(13),
            ],
        );

        let pipeline_layout =
            build::pipeline_layout(device, "PostProcess Pipeline Layout", &[&bind_group_layout]);

        let pipeline = build::Pipe::new(device, "PostProcess Pipeline", &shader)
            .layout(&pipeline_layout)
            .target(wgpu::ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
            .make();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let auto_exposure_shader = composer.compile_wgsl(
            device,
            "auto_exposure.wgsl",
            include_str!("../../shaders/auto_exposure.wgsl"),
        );

        let auto_exposure_bind_group_layout = build::bind_layout(
            device,
            "Auto Exposure Bind Group Layout",
            &[
                build::texture_at(
                    0,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::TextureViewDimension::D2,
                    wgpu::ShaderStages::COMPUTE,
                ),
                build::storage_at(1, true, wgpu::ShaderStages::COMPUTE),
            ],
        );

        let auto_exposure_pipeline_layout = build::pipeline_layout(
            device,
            "Auto Exposure Pipeline Layout",
            &[&auto_exposure_bind_group_layout],
        );

        let auto_exposure_pipeline = build::compute(
            device,
            "Auto Exposure Pipeline",
            &auto_exposure_shader,
            "main",
            &auto_exposure_pipeline_layout,
        );

        let auto_exposure_gather_shader = composer.compile_wgsl(
            device,
            "auto_exposure_gather.wgsl",
            include_str!("../../shaders/auto_exposure_gather.wgsl"),
        );

        let auto_exposure_gather_pipeline = build::compute(
            device,
            "Auto Exposure Gather Pipeline",
            &auto_exposure_gather_shader,
            "main",
            &auto_exposure_pipeline_layout,
        );

        let mut exposure_contents = vec![0u32; EXPOSURE_HEAD + EXPOSURE_STEPS];
        exposure_contents[2] = 1.0_f32.to_bits();
        let auto_exposure_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Auto Exposure Buffer"),
            contents: bytemuck::cast_slice(&exposure_contents),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            params_buffer,
            cached_bind_group: None,
            hdr_source_is_dof_output: false,
            letterbox_amount: 0.0,
            fallback_texture_view,
            auto_exposure_pipeline,
            auto_exposure_gather_pipeline,
            auto_exposure_bind_group_layout,
            auto_exposure_bind_group: None,
            auto_exposure_buffer,
            color_lut_texture,
            color_lut_view,
            color_lut_sampler,
            pending_color_lut: Some(identity_color_lut_data()),
            output_encodes_srgb: surface_format.is_srgb(),
        }
    }

    pub fn set_color_lut(&mut self, data: Vec<u8>) {
        self.pending_color_lut = Some(data);
    }
}

const POST_PROCESS_PASS_DECL: PassDecl = PassDecl {
    name: "postprocess_pass",
    position: PassPosition::Present,
    reads: &["hdr", "scene_color", "bloom", "ssao"],
    writes: &["output"],
    optional_reads: &["ssgi", "ssr"],
    ..PassDecl::EMPTY
};

impl PassNode<crate::renderer::wgpu::render_configs::RenderInputs> for PostProcessPass {
    fn decl(&self) -> &PassDecl {
        &POST_PROCESS_PASS_DECL
    }

    fn invalidate_bind_groups(&mut self) {
        self.cached_bind_group = None;
    }

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::renderer::wgpu::render_configs::RenderInputs,
    ) {
        self.letterbox_amount = configs.scene.letterbox_amount;
        let view = &configs.scene.active_view;
        let color_grading = &view.color_grading;
        let bloom_enabled = if view.bloom_enabled { 1.0 } else { 0.0 };
        let ssao_enabled = if view.ssao_enabled { 1.0 } else { 0.0 };
        let bloom_intensity = view.bloom_intensity;
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
            gamma: if self.output_encodes_srgb {
                1.0
            } else {
                color_grading.gamma
            },
            saturation: color_grading.saturation,
            brightness: color_grading.brightness,
            contrast: color_grading.contrast,
            tonemap_algorithm: color_grading.tonemap_algorithm.as_u32(),
            ssao_enabled,
            time: configs.view.uptime_milliseconds as f32 / 1000.0,
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
        context: PassExecutionContext<'r, 'e, crate::renderer::wgpu::render_configs::RenderInputs>,
    ) -> crate::renderer::wgpu::rendergraph::Result<()> {
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

            self.cached_bind_group = Some(build::binding(
                context.device,
                "PostProcess Bind Group",
                &self.bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(hdr_view),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    self.params_buffer.as_entire_binding(),
                    wgpu::BindingResource::TextureView(bloom_view),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    wgpu::BindingResource::TextureView(ssao_view),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    wgpu::BindingResource::TextureView(ssgi_view),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    wgpu::BindingResource::TextureView(ssr_view),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    self.auto_exposure_buffer.as_entire_binding(),
                    wgpu::BindingResource::TextureView(&self.color_lut_view),
                    wgpu::BindingResource::Sampler(&self.color_lut_sampler),
                ],
            ));

            self.auto_exposure_bind_group = Some(build::binding(
                context.device,
                "Auto Exposure Bind Group",
                &self.auto_exposure_bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(hdr_view),
                    self.auto_exposure_buffer.as_entire_binding(),
                ],
            ));
        }

        if context.configs.settings.color_grading.auto_exposure
            && let Some(bg) = self.auto_exposure_bind_group.as_ref()
        {
            let adaptation_rate = context.configs.settings.color_grading.auto_exposure_rate;
            let delta_time = context.configs.view.delta_time.clamp(1.0e-4, 1.0);
            context.queue.write_buffer(
                &self.auto_exposure_buffer,
                8,
                bytemuck::cast_slice(&[adaptation_rate, delta_time]),
            );
            context.queue.write_buffer(
                &self.auto_exposure_buffer,
                20,
                bytemuck::cast_slice(&[DARKEST_READING, BRIGHTEST_READING]),
            );

            let (width, height) = context
                .configs
                .scene
                .render_view
                .as_ref()
                .map(|view| view.screen_size)
                .or(context.configs.view.scene_target_size)
                .or(context.configs.view.viewport_size)
                .unwrap_or((1, 1));

            let mut compute_pass =
                context
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Auto Exposure Compute Pass"),
                        timestamp_writes: None,
                    });
            compute_pass.set_bind_group(0, bg, &[]);
            compute_pass.set_pipeline(&self.auto_exposure_gather_pipeline);
            compute_pass.dispatch_workgroups(
                width.div_ceil(EXPOSURE_GATHER_SPAN),
                height.div_ceil(EXPOSURE_GATHER_SPAN),
                1,
            );
            compute_pass.set_pipeline(&self.auto_exposure_pipeline);
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

        Ok(())
    }
}
