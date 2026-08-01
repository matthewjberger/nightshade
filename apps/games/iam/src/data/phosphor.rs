use crate::data::config::Config;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, PassStage, Result};
use nightshade_wgpu::wgpu;

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Settings {
    decay: f32,
    lift: f32,
    reach: f32,
    gain: f32,
}

pub struct PhosphorPass {
    blend: wgpu::RenderPipeline,
    copy: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    settings: Settings,
    format: wgpu::TextureFormat,
    trails: Option<[wgpu::TextureView; 2]>,
    size: (u32, u32),
    written: usize,
    blend_bound: Option<wgpu::BindGroup>,
    copy_bound: Option<wgpu::BindGroup>,
}

const CANVAS: &str = "canvas";

pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> PhosphorPass {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("phosphor"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/phosphor.wgsl").into()),
    });

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("phosphor"),
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
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
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
            wgpu::BindGroupLayoutEntry {
                binding: 3,
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
        label: Some("phosphor"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });

    let pipeline = |entry: &str, label: &str| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("fullscreenVertex"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some(entry),
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
        })
    };

    PhosphorPass {
        blend: pipeline("blendMain", "phosphor_blend"),
        copy: pipeline("copyMain", "phosphor_copy"),
        layout,
        sampler: device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("phosphor"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        }),
        uniforms: device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phosphor"),
            size: std::mem::size_of::<Settings>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        settings: Settings::default(),
        format: surface_format,
        trails: None,
        size: (0, 0),
        written: 0,
        blend_bound: None,
        copy_bound: None,
    }
}

pub fn configure(pass: &mut PhosphorPass, config: &Config) {
    pass.settings = Settings {
        decay: config.screen_persistence,
        lift: config.screen_persistence_lift,
        reach: config.screen_persistence_reach,
        gain: config.screen_persistence_gain,
    };
}

fn fit(pass: &mut PhosphorPass, device: &wgpu::Device, width: u32, height: u32) {
    if pass.size == (width, height) && pass.trails.is_some() {
        return;
    }
    let make = |label: &str| {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: pass.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    };
    pass.trails = Some([make("phosphor_a"), make("phosphor_b")]);
    pass.size = (width, height);
    pass.written = 0;
    pass.blend_bound = None;
    pass.copy_bound = None;
}

fn upload(
    pass: &mut PhosphorPass,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    configs: &RenderInputs,
) {
    queue.write_buffer(&pass.uniforms, 0, bytemuck::bytes_of(&pass.settings));
    let (width, height) = configs.view.viewport_size.unwrap_or((1, 1));
    fit(pass, device, width.max(1), height.max(1));
}

fn draw(
    pass: &mut PhosphorPass,
    context: PassExecutionContext<'_, '_, RenderInputs>,
) -> Result<()> {
    if pass.settings.decay <= 0.0 {
        return Ok(());
    }
    let Some(trails) = pass.trails.as_ref() else {
        return Ok(());
    };
    let canvas = context.get_texture_view(CANVAS)?;
    let writing = 1 - pass.written;

    if pass.blend_bound.is_none() {
        pass.blend_bound = Some(
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("phosphor_blend"),
                    layout: &pass.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(canvas),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&trails[pass.written]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&pass.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: pass.uniforms.as_entire_binding(),
                        },
                    ],
                }),
        );
    }
    if pass.copy_bound.is_none() {
        pass.copy_bound = Some(
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("phosphor_copy"),
                    layout: &pass.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&trails[pass.written]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&trails[writing]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&pass.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: pass.uniforms.as_entire_binding(),
                        },
                    ],
                }),
        );
    }

    {
        let mut render = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iam_phosphor_blend"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &trails[writing],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        if let Some(bound) = pass.blend_bound.as_ref() {
            render.set_pipeline(&pass.blend);
            render.set_bind_group(0, bound, &[]);
            render.draw(0..3, 0..1);
        }
    }

    let (view, load, store) = context.get_color_attachment(CANVAS)?;
    let view = view.clone();
    {
        let mut render = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("iam_phosphor_copy"),
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
        if let Some(bound) = pass.copy_bound.as_ref() {
            render.set_pipeline(&pass.copy);
            render.set_bind_group(0, bound, &[]);
            render.draw(0..3, 0..1);
        }
    }

    pass.written = writing;
    pass.blend_bound = None;
    pass.copy_bound = None;
    Ok(())
}

const PHOSPHOR_PASS_DECL: PassDecl = PassDecl {
    name: "iam_phosphor",
    reads_writes: &[CANVAS],
    stage: PassStage::Swapchain,
    ..PassDecl::EMPTY
};

impl PassNode<RenderInputs> for PhosphorPass {
    fn decl(&self) -> &PassDecl {
        &PHOSPHOR_PASS_DECL
    }

    fn invalidate_bind_groups(&mut self) {
        self.blend_bound = None;
        self.copy_bound = None;
    }

    fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, configs: &RenderInputs) {
        upload(self, device, queue, configs);
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> Result<()> {
        draw(self, context)
    }
}
