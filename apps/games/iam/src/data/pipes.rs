use crate::data::config::Config;
use nightshade::prelude::*;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, PassStage, Result};
use nightshade_wgpu::wgpu;

pub const SEGMENTS: usize = 64;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Settings {
    frame: [f32; 4],
    room: [f32; 4],
    low: [f32; 4],
    high: [f32; 4],
    heads: [[f32; 4]; SEGMENTS],
    tails: [[f32; 4]; SEGMENTS],
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            frame: [0.0; 4],
            room: [1.0, 1.0, 0.09, 1.0],
            low: [1.0, 1.0, 1.0, 0.0],
            high: [-1.0, -1.0, -1.0, 0.0],
            heads: [[0.0; 4]; SEGMENTS],
            tails: [[0.0; 4]; SEGMENTS],
        }
    }
}

pub struct PipesPass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniforms: wgpu::Buffer,
    bound: Option<wgpu::BindGroup>,
    settings: Settings,
    cover: f32,
}

const CANVAS: &str = "canvas";

pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> PipesPass {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("pipes"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/pipes.wgsl").into()),
    });

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("pipes"),
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
        label: Some("pipes"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("pipes"),
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

    PipesPass {
        pipeline,
        layout,
        uniforms: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pipes"),
            size: std::mem::size_of::<Settings>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        bound: None,
        settings: Settings::default(),
        cover: 0.0,
    }
}

pub fn configure(pass: &mut PipesPass, config: &Config) {
    pass.settings.room[2] = config.pipes_radius;
    pass.settings.room[3] = config.pipes_room_light;
}

pub fn set_run(pass: &mut PipesPass, cover: f32, grown: f32, run: &[Segment]) {
    pass.cover = cover;
    pass.settings.frame[2] = run.len().min(SEGMENTS) as f32;
    pass.settings.frame[3] = grown;
    let mut low = [f32::MAX; 3];
    let mut high = [f32::MIN; 3];
    for (slot, segment) in run.iter().take(SEGMENTS).enumerate() {
        pass.settings.heads[slot] = [
            segment.head.x,
            segment.head.y,
            segment.head.z,
            segment.enamel as f32,
        ];
        pass.settings.tails[slot] = [segment.tail.x, segment.tail.y, segment.tail.z, 0.0];
        for axis in 0..3 {
            low[axis] = low[axis].min(pass.settings.heads[slot][axis]);
            low[axis] = low[axis].min(pass.settings.tails[slot][axis]);
            high[axis] = high[axis].max(pass.settings.heads[slot][axis]);
            high[axis] = high[axis].max(pass.settings.tails[slot][axis]);
        }
    }
    let margin = pass.settings.room[2] * 1.4;
    for axis in 0..3 {
        pass.settings.low[axis] = low[axis] - margin;
        pass.settings.high[axis] = high[axis] + margin;
    }
}

#[derive(Clone, Copy, Default)]
pub struct Segment {
    pub head: Vec3,
    pub tail: Vec3,
    pub enamel: u32,
}

fn upload(pass: &mut PipesPass, queue: &wgpu::Queue, configs: &RenderInputs) {
    let mut settings = pass.settings;
    settings.frame[0] = configs.view.uptime_milliseconds as f32 * 0.001;
    settings.frame[1] = pass.cover;
    let (width, height) = configs.view.viewport_size.unwrap_or((1, 1));
    settings.room[0] = width.max(1) as f32;
    settings.room[1] = height.max(1) as f32;
    queue.write_buffer(&pass.uniforms, 0, bytemuck::bytes_of(&settings));
}

fn draw(pass: &mut PipesPass, context: PassExecutionContext<'_, '_, RenderInputs>) -> Result<()> {
    if pass.cover <= 0.0 {
        return Ok(());
    }
    if pass.bound.is_none() {
        pass.bound = Some(
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("pipes"),
                    layout: &pass.layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: pass.uniforms.as_entire_binding(),
                    }],
                }),
        );
    }

    let (view, load, store) = context.get_color_attachment(CANVAS)?;
    let view = view.clone();
    let mut render = context
        .encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("iam_pipes"),
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

const PIPES_PASS_DECL: PassDecl = PassDecl {
    name: "iam_pipes",
    reads_writes: &[CANVAS],
    stage: PassStage::Swapchain,
    ..PassDecl::EMPTY
};

impl PassNode<RenderInputs> for PipesPass {
    fn decl(&self) -> &PassDecl {
        &PIPES_PASS_DECL
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
