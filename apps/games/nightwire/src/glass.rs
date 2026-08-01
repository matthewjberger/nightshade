use crate::resources::NightwireResources;
use nightshade::prelude::Res;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::{
    PassDecl, PassExecutionContext, PassNode, PassPosition, PassStage, Result,
    render_graph_add_color_texture, render_graph_add_pass_at, render_graph_bind_pass_slot,
    render_graph_get_pass_mut,
};
use nightshade_wgpu::pre_render::Renderer;
use nightshade_wgpu::wgpu;

const PASS_NAME: &str = "nightwire_glass";
const TEXTURE_NAME: &str = "nightwire_interface";
const INPUT: &str = "input";
const OUTPUT: &str = "output";
const INTERFACE_PASSES: [&str; 3] = ["viewport_compose_pass", "paint_pass", "paint_image_pass"];

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Glass {
    time: f32,
    amount: f32,
    curvature: f32,
    vignette: f32,
    scanlines: f32,
    scanline_pitch: f32,
    grille: f32,
    bleed: f32,
    warm: f32,
    flicker: f32,
    noise: f32,
    gain: f32,
}

impl Default for Glass {
    fn default() -> Self {
        Self {
            time: 0.0,
            amount: 1.0,
            curvature: 0.16,
            vignette: 0.20,
            scanlines: 0.10,
            scanline_pitch: std::f32::consts::PI,
            grille: 0.05,
            bleed: 0.6,
            warm: 0.22,
            flicker: 0.014,
            noise: 0.018,
            gain: 1.12,
        }
    }
}

pub struct GlassPass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    bound: Option<wgpu::BindGroup>,
    settings: Glass,
}

fn new_pass(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> GlassPass {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(PASS_NAME),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/crt.wgsl").into()),
    });

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(PASS_NAME),
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
        label: Some(PASS_NAME),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(PASS_NAME),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("fullscreen_vertex"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fragment_main"),
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

    GlassPass {
        pipeline,
        layout,
        sampler: device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(PASS_NAME),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }),
        uniforms: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(PASS_NAME),
            size: std::mem::size_of::<Glass>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        bound: None,
        settings: Glass::default(),
    }
}

fn upload(pass: &mut GlassPass, queue: &wgpu::Queue, configs: &RenderInputs) {
    let mut settings = pass.settings;
    settings.time = configs.view.uptime_milliseconds as f32 * 0.001;
    queue.write_buffer(&pass.uniforms, 0, bytemuck::bytes_of(&settings));
}

fn draw(pass: &mut GlassPass, context: PassExecutionContext<'_, '_, RenderInputs>) -> Result<()> {
    let input = context.get_texture_view(INPUT)?;
    if pass.bound.is_none() {
        pass.bound = Some(
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(PASS_NAME),
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
            label: Some(PASS_NAME),
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

const GLASS_PASS_DECL: PassDecl = PassDecl {
    name: PASS_NAME,
    reads: &[INPUT],
    writes: &[OUTPUT],
    stage: PassStage::Swapchain,
    ..PassDecl::EMPTY
};

impl PassNode<RenderInputs> for GlassPass {
    fn decl(&self) -> &PassDecl {
        &GLASS_PASS_DECL
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

pub fn install(mut renderer: Renderer) {
    if render_graph_get_pass_mut(&mut renderer.graph, PASS_NAME).is_some() {
        return;
    }
    let drawn = INTERFACE_PASSES
        .iter()
        .any(|pass| render_graph_get_pass_mut(&mut renderer.graph, pass).is_some());
    if !drawn {
        return;
    }

    let format = renderer.surface_format;
    let width = renderer.surface_config.width.max(1);
    let height = renderer.surface_config.height.max(1);
    let interface = render_graph_add_color_texture(&mut renderer.graph, TEXTURE_NAME)
        .format(format)
        .size(width, height)
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING)
        .clear_color(wgpu::Color::BLACK)
        .transient();

    for pass in INTERFACE_PASSES {
        let slot = if pass == "viewport_compose_pass" {
            "output"
        } else {
            "color"
        };
        if render_graph_get_pass_mut(&mut renderer.graph, pass).is_some()
            && let Err(error) =
                render_graph_bind_pass_slot(&mut renderer.graph, pass, slot, interface)
        {
            panic!("nightwire could not retarget {pass}: {error}");
        }
    }

    let swapchain = renderer.targets.swapchain;
    let pass = new_pass(&renderer.device, format);
    render_graph_add_pass_at(
        &mut renderer.graph,
        Box::new(pass),
        &[(INPUT, interface), (OUTPUT, swapchain)],
        PassPosition::Last,
    )
    .expect("the glass pass");
}

pub fn drive(mut renderer: Renderer, resources: Res<NightwireResources>) {
    let amount = resources.glass_amount;
    let Some(pass) = render_graph_get_pass_mut(&mut renderer.graph, PASS_NAME) else {
        return;
    };
    let Some(glass) = (pass as &mut dyn std::any::Any).downcast_mut::<GlassPass>() else {
        return;
    };
    glass.settings.amount = amount;
}
