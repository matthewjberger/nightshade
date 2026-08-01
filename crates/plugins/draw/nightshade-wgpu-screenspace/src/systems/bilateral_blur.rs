use nightshade_wgpu::backend::EffectiveShading;
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, PassPosition};
use nightshade_wgpu::wgpu;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurParams {
    screen_size: [f32; 2],
    depth_threshold: f32,
    normal_param: f32,
}

pub struct BilateralBlurSpec {
    pub decl: PassDecl,
    pub label: &'static str,
    pub source: &'static str,
    pub output: &'static str,
    pub format: wgpu::TextureFormat,
    pub enabled: fn(&EffectiveShading) -> bool,
    pub thresholds: fn(&EffectiveShading) -> (f32, f32),
}

pub struct BilateralBlurPass {
    enabled_when: Option<fn(&nightshade_wgpu::backend::RenderInputs) -> bool>,
    decl: PassDecl,
    spec: BilateralBlurSpec,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    linear_sampler: wgpu::Sampler,
    point_sampler: wgpu::Sampler,
    params_buffer: wgpu::Buffer,
    cached_bind_group: Option<wgpu::BindGroup>,
    cached_params: BlurParams,
}

const SSAO_BLUR_DECL: PassDecl = PassDecl {
    name: "ssao_blur_pass",
    position: PassPosition::PostProcess,
    reads: &["ssao_raw", "depth", "view_normals"],
    writes: &["ssao"],
    ..PassDecl::EMPTY
};

const SSGI_BLUR_DECL: PassDecl = PassDecl {
    name: "ssgi_blur_pass",
    position: PassPosition::PostProcess,
    reads: &["ssgi_raw", "depth", "view_normals"],
    writes: &["ssgi"],
    ..PassDecl::EMPTY
};

const SSR_BLUR_DECL: PassDecl = PassDecl {
    name: "ssr_blur_pass",
    position: PassPosition::PostProcess,
    reads: &["ssr_raw", "depth", "view_normals"],
    writes: &["ssr"],
    ..PassDecl::EMPTY
};

impl BilateralBlurPass {
    pub fn when(mut self, predicate: fn(&nightshade_wgpu::backend::RenderInputs) -> bool) -> Self {
        self.enabled_when = Some(predicate);
        self
    }
    pub fn ssao(build: PassBuild) -> Self {
        Self::new(
            build,
            BilateralBlurSpec {
                decl: SSAO_BLUR_DECL,
                label: "SSAO",
                source: "ssao_raw",
                output: "ssao",
                format: wgpu::TextureFormat::R8Unorm,
                enabled: |view| view.ssao_enabled,
                thresholds: |view| (view.ssao_blur_depth_threshold, view.ssao_blur_normal_power),
            },
            "ssao_blur.wgsl",
            include_str!("../shaders/ssao_blur.wgsl"),
        )
    }

    pub fn ssgi(build: PassBuild) -> Self {
        Self::new(
            build,
            BilateralBlurSpec {
                decl: SSGI_BLUR_DECL,
                label: "SSGI",
                source: "ssgi_raw",
                output: "ssgi",
                format: wgpu::TextureFormat::Rgba16Float,
                enabled: |view| view.ssgi_enabled,
                thresholds: |_| (0.05, 16.0),
            },
            "ssgi_blur.wgsl",
            include_str!("../shaders/ssgi_blur.wgsl"),
        )
    }

    pub fn ssr(build: PassBuild) -> Self {
        Self::new(
            build,
            BilateralBlurSpec {
                decl: SSR_BLUR_DECL,
                label: "SSR",
                source: "ssr_raw",
                output: "ssr",
                format: wgpu::TextureFormat::Rgba16Float,
                enabled: |view| view.ssr_enabled,
                thresholds: |_| (0.05, 16.0),
            },
            "ssr_blur.wgsl",
            include_str!("../shaders/ssr_blur.wgsl"),
        )
    }

    fn new(
        build: PassBuild,
        spec: BilateralBlurSpec,
        shader_name: &str,
        shader_source: &str,
    ) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let shader = composer.compile_wgsl(device, shader_name, shader_source);

        let (depth_threshold, normal_param) = (spec.thresholds)(&EffectiveShading::default());
        let params = BlurParams {
            screen_size: [1920.0, 1080.0],
            depth_threshold,
            normal_param,
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Blur Params Buffer", spec.label)),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = build::bind_layout(
            device,
            &format!("{} Blur Bind Group Layout", spec.label),
            &[
                build::painted_at(0),
                build::depth_at(1, wgpu::ShaderStages::FRAGMENT),
                build::painted_at(2),
                build::filtering_at(3),
                build::sampler_at(
                    4,
                    wgpu::SamplerBindingType::NonFiltering,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::uniform_at(5, wgpu::ShaderStages::FRAGMENT),
            ],
        );

        let pipeline_layout = build::pipeline_layout(
            device,
            &format!("{} Blur Pipeline Layout", spec.label),
            &[&bind_group_layout],
        );

        let pipeline = build::Pipe::new(device, &format!("{} Blur Pipeline", spec.label), &shader)
            .layout(&pipeline_layout)
            .target(wgpu::ColorTargetState {
                format: spec.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })
            .make();

        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
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

        Self {
            enabled_when: None,
            decl: spec.decl,
            spec,
            pipeline,
            bind_group_layout,
            linear_sampler,
            point_sampler,
            params_buffer,
            cached_bind_group: None,
            cached_params: params,
        }
    }
}

impl PassNode<nightshade_wgpu::backend::RenderInputs> for BilateralBlurPass {
    fn decl(&self) -> &PassDecl {
        &self.decl
    }

    fn enabled(&self, configs: &nightshade_wgpu::backend::RenderInputs) -> bool {
        self.enabled_when.is_none_or(|predicate| predicate(configs))
    }

    fn invalidate_bind_groups(&mut self) {
        self.cached_bind_group = None;
    }

    fn prepare(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        let (screen_width, screen_height) = configs.view.viewport_size.unwrap_or((1920, 1080));
        let (depth_threshold, normal_param) = (self.spec.thresholds)(&configs.scene.active_view);

        let params = BlurParams {
            screen_size: [
                (screen_width / 2).max(1) as f32,
                (screen_height / 2).max(1) as f32,
            ],
            depth_threshold,
            normal_param,
        };

        if params != self.cached_params {
            queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[params]));
            self.cached_params = params;
        }
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, nightshade_wgpu::backend::RenderInputs>,
    ) -> nightshade_wgpu::backend::Result<()> {
        if !context.is_pass_enabled() {
            return Ok(());
        }
        if !(self.spec.enabled)(&context.configs.scene.active_view) {
            return Ok(());
        }

        if self.cached_bind_group.is_none() {
            let source_view = context.get_texture_view(self.spec.source)?;
            let (depth_view, _, _) = context.get_depth_attachment("depth")?;
            let normal_view = context.get_texture_view("view_normals")?;

            self.cached_bind_group = Some(build::binding(
                context.device,
                &format!("{} Blur Bind Group", self.spec.label),
                &self.bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(source_view),
                    wgpu::BindingResource::TextureView(depth_view),
                    wgpu::BindingResource::TextureView(normal_view),
                    wgpu::BindingResource::Sampler(&self.linear_sampler),
                    wgpu::BindingResource::Sampler(&self.point_sampler),
                    self.params_buffer.as_entire_binding(),
                ],
            ));
        }

        let (output_view, output_load_op, output_store_op) =
            context.get_color_attachment(self.spec.output)?;

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("{} Blur Render Pass", self.spec.label)),
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

        Ok(())
    }
}
