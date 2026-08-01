use crate::data;
use nightshade::prelude::{Entity, Mat4};
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, Result};
use nightshade_wgpu::wgpu;

pub const MAX_PORTALS: usize = 2;
const UNIFORM_SIZE: u64 = 96;
const COLOR: &str = "color";
const DEPTH: &str = "depth";

pub struct PortalDraw {
    pub model: Mat4,
    pub view: wgpu::TextureView,
    pub rim: [f32; 4],
    pub fade: f32,
}

pub struct PortalPass {
    shader: wgpu::ShaderModule,
    layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: Option<wgpu::RenderPipeline>,
    target: Option<(wgpu::TextureFormat, u32)>,
    sampler: wgpu::Sampler,
    uniforms: Vec<wgpu::Buffer>,
    draws: Vec<PortalDraw>,
    player_camera: Entity,
    elapsed: f32,
}

pub fn new(build: PassBuild) -> PortalPass {
    let device = build.device;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(data::PORTAL_PASS),
        source: wgpu::ShaderSource::Wgsl(include_str!("portal.wgsl").into()),
    });

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(data::PORTAL_PASS),
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
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
        label: Some(data::PORTAL_PASS),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });

    let uniforms = (0..MAX_PORTALS)
        .map(|slot| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("{} {slot}", data::PORTAL_PASS)),
                size: UNIFORM_SIZE,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        })
        .collect();

    PortalPass {
        shader,
        layout,
        pipeline_layout,
        pipeline: None,
        target: None,
        sampler: device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(data::PORTAL_PASS),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }),
        uniforms,
        draws: Vec::new(),
        player_camera: Entity::default(),
        elapsed: 0.0,
    }
}

pub fn hand_over(
    pass: &mut PortalPass,
    player_camera: Entity,
    elapsed: f32,
    mut draws: Vec<PortalDraw>,
) {
    draws.truncate(MAX_PORTALS);
    pass.player_camera = player_camera;
    pass.elapsed = elapsed;
    pass.draws = draws;
}

fn columns(matrix: &Mat4, rim: [f32; 4], elapsed: f32, fade: f32) -> [f32; 24] {
    let mut flat = [0.0_f32; 24];
    for column in 0..4 {
        for row in 0..4 {
            flat[column * 4 + row] = matrix[(row, column)];
        }
    }
    flat[16..20].copy_from_slice(&rim);
    flat[20] = elapsed;
    flat[21] = fade;
    flat
}

fn view_projection(configs: &RenderInputs) -> Mat4 {
    let held = configs.scene.view_projection;
    let mut flat = [0.0_f32; 16];
    for column in 0..4 {
        for row in 0..4 {
            flat[column * 4 + row] = held[column][row];
        }
    }
    Mat4::from_column_slice(&flat)
}

fn build_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    target: (wgpu::TextureFormat, u32),
) -> wgpu::RenderPipeline {
    let (format, sample_count) = target;
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(data::PORTAL_PASS),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vertex_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fragment_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: sample_count,
            ..Default::default()
        },
        multiview_mask: None,
        cache: None,
    })
}

fn draw(
    pass: &mut PortalPass,
    context: &mut PassExecutionContext<'_, '_, RenderInputs>,
) -> Result<()> {
    if pass.draws.is_empty() || pass.player_camera == Entity::default() {
        return Ok(());
    }
    if context.configs.view.active_camera != Some(pass.player_camera) {
        return Ok(());
    }

    let color = context.get_texture(COLOR)?;
    let target = (color.format(), color.sample_count());
    if pass.target != Some(target) {
        pass.pipeline = Some(build_pipeline(
            context.device,
            &pass.pipeline_layout,
            &pass.shader,
            target,
        ));
        pass.target = Some(target);
    }
    let Some(pipeline) = pass.pipeline.as_ref() else {
        return Ok(());
    };

    let camera = view_projection(context.configs);
    let mut bound = Vec::with_capacity(pass.draws.len());
    for (slot, portal) in pass.draws.iter().enumerate() {
        let flat = columns(
            &(camera * portal.model),
            portal.rim,
            pass.elapsed,
            portal.fade,
        );
        context
            .queue
            .write_buffer(&pass.uniforms[slot], 0, bytemuck::cast_slice(&flat));
        bound.push(
            context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(data::PORTAL_PASS),
                    layout: &pass.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&portal.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&pass.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: pass.uniforms[slot].as_entire_binding(),
                        },
                    ],
                }),
        );
    }

    let (color_view, color_load, color_store) = context.get_color_attachment(COLOR)?;
    let (depth_view, depth_load, depth_store) = context.get_depth_attachment(DEPTH)?;
    let mut render_pass = context
        .encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(data::PORTAL_PASS),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: color_load,
                    store: color_store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: depth_load,
                    store: depth_store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

    render_pass.set_pipeline(pipeline);
    for group in &bound {
        render_pass.set_bind_group(0, group, &[]);
        render_pass.draw(0..6, 0..1);
    }
    Ok(())
}

const PORTAL_PASS_DECL: PassDecl = PassDecl {
    name: data::PORTAL_PASS,
    reads_writes: &[COLOR, DEPTH],
    ..PassDecl::EMPTY
};

impl PassNode<RenderInputs> for PortalPass {
    fn decl(&self) -> &PassDecl {
        &PORTAL_PASS_DECL
    }

    fn execute<'r, 'e>(
        &mut self,
        mut context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> Result<()> {
        draw(self, &mut context)
    }
}
