use crate::data::config::Config;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, PassStage, Result};
use nightshade_wgpu::wgpu;

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Screen {
    time: f32,
    curvature: f32,
    vignette: f32,
    corner: f32,
    falloff: f32,
    scanlines: f32,
    grille: f32,
    scanline_pitch: f32,
    grille_pitch: f32,
    bleed: f32,
    gain: f32,
    warm: f32,
    bezel: f32,
    bezel_depth: f32,
    bezel_highlight: f32,
    bezel_sheen: f32,
    bezel_radius: f32,
    bezel_mirror: f32,
    room_light: f32,
    lamp: f32,
    convergence: f32,
    reflection: f32,
    grime: f32,
    triad: f32,
    lens: f32,
    lens_x: f32,
    lens_y: f32,
    lens_radius: f32,
    lens_zoom: f32,
    shutoff: f32,
    pad_one: f32,
    pad_two: f32,
}

pub fn set_wear(pass: &mut ScreenPass, config: &Config, through: f32) {
    let held = through.clamp(0.0, 1.0);
    pass.settings.grime = config.screen_grime + config.wear_grime * held;
    pass.settings.convergence = config.screen_convergence + config.wear_convergence * held;
    pass.settings.vignette = config.screen_vignette + config.wear_vignette * held;
    pass.settings.reflection = config.screen_reflection + config.wear_reflection * held;
    pass.settings.gain = config.screen_gain - config.wear_gain * held;
    pass.settings.warm = config.screen_warm - config.wear_warm * held;
}

pub fn set_shutoff(pass: &mut ScreenPass, taken: f32) {
    pass.settings.shutoff = taken;
}

pub fn set_lens(pass: &mut ScreenPass, held: f32, at: nightshade::prelude::Vec2, zoom: f32) {
    pass.settings.lens = held;
    pass.settings.lens_x = at.x;
    pass.settings.lens_y = at.y;
    pass.settings.lens_zoom = zoom;
}

pub struct ScreenPass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    bound: Option<wgpu::BindGroup>,
    settings: Screen,
}

const INPUT: &str = "input";
const OUTPUT: &str = "output";

pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> ScreenPass {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("crt"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/crt.wgsl").into()),
    });

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("crt"),
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

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("crt"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("crt"),
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
                blend: None,
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

    ScreenPass {
        pipeline,
        layout,
        sampler: device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("crt"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }),
        uniforms: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("crt"),
            size: std::mem::size_of::<Screen>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        bound: None,
        settings: Screen::default(),
    }
}

pub fn configure(pass: &mut ScreenPass, config: &Config) {
    pass.settings = Screen {
        time: 0.0,
        curvature: config.screen_curvature,
        vignette: config.screen_vignette,
        corner: config.screen_corner,
        falloff: config.screen_falloff,
        scanlines: config.screen_scanlines,
        grille: config.screen_grille,
        scanline_pitch: config.screen_scanline_pitch,
        grille_pitch: config.screen_grille_pitch,
        bleed: config.screen_bleed,
        gain: config.screen_gain,
        warm: config.screen_warm,
        bezel: config.screen_bezel,
        bezel_depth: config.screen_bezel_depth,
        bezel_highlight: config.screen_bezel_highlight,
        bezel_sheen: config.screen_bezel_sheen,
        bezel_radius: config.screen_bezel_radius,
        bezel_mirror: config.screen_bezel_mirror,
        room_light: config.screen_room_light,
        lamp: config.screen_lamp,
        convergence: config.screen_convergence,
        reflection: config.screen_reflection,
        grime: config.screen_grime,
        triad: config.screen_triad,
        lens: 0.0,
        lens_x: 0.5,
        lens_y: 0.5,
        lens_radius: config.lens_radius,
        lens_zoom: config.lens_zoom,
        shutoff: 0.0,
        pad_one: 0.0,
        pad_two: 0.0,
    };
}

fn upload(pass: &mut ScreenPass, queue: &wgpu::Queue, configs: &RenderInputs) {
    let mut settings = pass.settings;
    settings.time = configs.view.uptime_milliseconds as f32 * 0.001;
    queue.write_buffer(&pass.uniforms, 0, bytemuck::bytes_of(&settings));
}

fn draw(pass: &mut ScreenPass, context: PassExecutionContext<'_, '_, RenderInputs>) -> Result<()> {
    let input = context.get_texture_view(INPUT)?;
    if pass.bound.is_none() {
        pass.bound = Some(
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("crt"),
                    layout: &pass.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(input),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&pass.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: pass.uniforms.as_entire_binding(),
                        },
                    ],
                }),
        );
    }

    let (view, load, store) = context.get_color_attachment(OUTPUT)?;
    let view = view.clone();
    let mut render = context
        .encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("iam_screen"),
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

const SCREEN_PASS_DECL: PassDecl = PassDecl {
    name: "iam_screen",
    reads: &[INPUT],
    writes: &[OUTPUT],
    stage: PassStage::Swapchain,
    ..PassDecl::EMPTY
};

impl PassNode<RenderInputs> for ScreenPass {
    fn decl(&self) -> &PassDecl {
        &SCREEN_PASS_DECL
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
