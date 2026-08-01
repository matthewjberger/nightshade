use crate::data::config::Config;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, PassStage, Result};
use nightshade_wgpu::wgpu;

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Settings {
    time: f32,
    width: f32,
    height: f32,
    darkness: f32,
    cover: f32,
    glitch: f32,
    sun: f32,
    grid_speed: f32,
}

pub struct IntroPass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniforms: wgpu::Buffer,
    bound: Option<wgpu::BindGroup>,
    settings: Settings,
    darkness: f32,
    cover: f32,
}

const TARGET: &str = "canvas";

pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> IntroPass {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("intro"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/intro.wgsl").into()),
    });

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("intro"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("intro"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("intro"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("fullscreenVertex"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fragmentMain"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    IntroPass {
        pipeline,
        layout,
        uniforms: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("intro"),
            size: std::mem::size_of::<Settings>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        bound: None,
        settings: Settings::default(),
        darkness: 1.0,
        cover: 1.0,
    }
}

pub fn configure(pass: &mut IntroPass, config: &Config) {
    pass.settings = Settings {
        time: 0.0,
        width: 1.0,
        height: 1.0,
        darkness: 1.0,
        cover: 1.0,
        glitch: config.intro_glitch,
        sun: config.intro_sun,
        grid_speed: config.intro_grid_speed,
    };
}

pub fn set_fade(pass: &mut IntroPass, darkness: f32, cover: f32) {
    pass.darkness = darkness;
    pass.cover = cover;
}

fn upload(pass: &mut IntroPass, queue: &wgpu::Queue, configs: &RenderInputs) {
    let mut settings = pass.settings;
    settings.time = configs.view.uptime_milliseconds as f32 * 0.001;
    let (width, height) = configs.view.viewport_size.unwrap_or((1, 1));
    settings.width = width.max(1) as f32;
    settings.height = height.max(1) as f32;
    settings.darkness = pass.darkness;
    settings.cover = pass.cover;
    queue.write_buffer(&pass.uniforms, 0, bytemuck::bytes_of(&settings));
}

fn draw(pass: &mut IntroPass, context: PassExecutionContext<'_, '_, RenderInputs>) -> Result<()> {
    if pass.cover <= 0.0 {
        return Ok(());
    }
    if pass.bound.is_none() {
        pass.bound = Some(
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("intro"),
                    layout: &pass.layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: pass.uniforms.as_entire_binding(),
                    }],
                }),
        );
    }

    let (view, load, store) = context.get_color_attachment(TARGET)?;
    let view = view.clone();
    let mut render = context
        .encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("iam_intro"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations { load, store },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    if let Some(bound) = pass.bound.as_ref() {
        render.set_pipeline(&pass.pipeline);
        render.set_bind_group(0, bound, &[]);
        render.draw(0..3, 0..1);
    }
    Ok(())
}

const INTRO_PASS_DECL: PassDecl = PassDecl {
    name: "iam_intro",
    reads_writes: &[TARGET],
    stage: PassStage::Swapchain,
    ..PassDecl::EMPTY
};

impl PassNode<RenderInputs> for IntroPass {
    fn decl(&self) -> &PassDecl {
        &INTRO_PASS_DECL
    }

    fn invalidate_bind_groups(&mut self) {
        self.bound = None;
    }

    fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, configs: &RenderInputs) {
        upload(self, queue, configs);
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> Result<()> {
        draw(self, context)
    }
}
