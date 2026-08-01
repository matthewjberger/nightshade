use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, Result};

pub const CURTAIN_PASS: &str = "blocker_curtain_pass";

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CurtainUniforms {
    time: f32,
    aspect: f32,
    progress: f32,
    seed: f32,
    width: f32,
    height: f32,
    steps: f32,
    pad: f32,
}

pub struct CurtainPass {
    shader: nightshade_wgpu::wgpu::ShaderModule,
    layout: nightshade_wgpu::wgpu::PipelineLayout,
    uniforms: nightshade_wgpu::wgpu::Buffer,
    bind_group: nightshade_wgpu::wgpu::BindGroup,
    pipeline: Option<nightshade_wgpu::wgpu::RenderPipeline>,
    target: Option<(nightshade_wgpu::wgpu::TextureFormat, u32)>,
    pub time: f32,
    pub progress: f32,
    pub seed: f32,
    pub shown: bool,
}

impl CurtainPass {
    pub fn new(build: nightshade_wgpu::backend::PassBuild<'_>) -> Self {
        let nightshade_wgpu::backend::PassBuild {
            device, composer, ..
        } = build;
        let shader =
            composer.compile_wgsl(device, "curtain.wgsl", include_str!("shaders/curtain.wgsl"));
        let bind_layout =
            device.create_bind_group_layout(&nightshade_wgpu::wgpu::BindGroupLayoutDescriptor {
                label: Some("Curtain Bind Layout"),
                entries: &[nightshade_wgpu::wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: nightshade_wgpu::wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: nightshade_wgpu::wgpu::BindingType::Buffer {
                        ty: nightshade_wgpu::wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let layout =
            device.create_pipeline_layout(&nightshade_wgpu::wgpu::PipelineLayoutDescriptor {
                label: Some("Curtain Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_layout)],
                immediate_size: 0,
            });
        let uniforms = device.create_buffer(&nightshade_wgpu::wgpu::BufferDescriptor {
            label: Some("Curtain Uniforms"),
            size: std::mem::size_of::<CurtainUniforms>() as u64,
            usage: nightshade_wgpu::wgpu::BufferUsages::UNIFORM
                | nightshade_wgpu::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&nightshade_wgpu::wgpu::BindGroupDescriptor {
            label: Some("Curtain Bind Group"),
            layout: &bind_layout,
            entries: &[nightshade_wgpu::wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });
        Self {
            shader,
            layout,
            uniforms,
            bind_group,
            pipeline: None,
            target: None,
            time: 0.0,
            progress: 0.0,
            seed: 0.0,
            shown: false,
        }
    }
}

const CURTAIN_PASS_DECL: PassDecl = PassDecl {
    name: CURTAIN_PASS,
    reads_writes: &["color", "depth"],
    ..PassDecl::EMPTY
};

impl PassNode<RenderInputs> for CurtainPass {
    fn decl(&self) -> &PassDecl {
        &CURTAIN_PASS_DECL
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> Result<()> {
        if !self.shown {
            return Ok(());
        }
        let color = context.get_texture("color")?;
        let target = (color.format(), color.sample_count());
        if self.target != Some(target) {
            self.pipeline = Some(build_pipeline(
                context.device,
                &self.layout,
                &self.shader,
                target,
            ));
            self.target = Some(target);
        }
        let Some(pipeline) = self.pipeline.as_ref() else {
            return Ok(());
        };

        let (width, height) = context.get_texture_size("color")?;
        let uniforms = CurtainUniforms {
            time: self.time,
            aspect: width as f32 / height.max(1) as f32,
            progress: self.progress,
            seed: self.seed,
            width: width as f32,
            height: height as f32,
            steps: crate::theme::CURTAIN_STEPS,
            pad: 0.0,
        };
        context
            .queue
            .write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&uniforms));

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;
        let mut render_pass =
            context
                .encoder
                .begin_render_pass(&nightshade_wgpu::wgpu::RenderPassDescriptor {
                    label: Some("Curtain Pass"),
                    color_attachments: &[Some(nightshade_wgpu::wgpu::RenderPassColorAttachment {
                        view: color_view,
                        resolve_target: None,
                        ops: nightshade_wgpu::wgpu::Operations {
                            load: color_load,
                            store: color_store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: Some(
                        nightshade_wgpu::wgpu::RenderPassDepthStencilAttachment {
                            view: depth_view,
                            depth_ops: Some(nightshade_wgpu::wgpu::Operations {
                                load: depth_load,
                                store: depth_store,
                            }),
                            stencil_ops: None,
                        },
                    ),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
        Ok(())
    }
}

fn build_pipeline(
    device: &nightshade_wgpu::wgpu::Device,
    layout: &nightshade_wgpu::wgpu::PipelineLayout,
    shader: &nightshade_wgpu::wgpu::ShaderModule,
    target: (nightshade_wgpu::wgpu::TextureFormat, u32),
) -> nightshade_wgpu::wgpu::RenderPipeline {
    let (format, sample_count) = target;
    device.create_render_pipeline(&nightshade_wgpu::wgpu::RenderPipelineDescriptor {
        label: Some("Curtain Pipeline"),
        layout: Some(layout),
        vertex: nightshade_wgpu::wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(nightshade_wgpu::wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(nightshade_wgpu::wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: nightshade_wgpu::wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: nightshade_wgpu::wgpu::PrimitiveState {
            topology: nightshade_wgpu::wgpu::PrimitiveTopology::TriangleList,
            front_face: nightshade_wgpu::wgpu::FrontFace::Ccw,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(nightshade_wgpu::wgpu::DepthStencilState {
            format: nightshade_wgpu::wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: Some(false),
            depth_compare: Some(nightshade_wgpu::wgpu::CompareFunction::Always),
            stencil: nightshade_wgpu::wgpu::StencilState::default(),
            bias: nightshade_wgpu::wgpu::DepthBiasState::default(),
        }),
        multisample: nightshade_wgpu::wgpu::MultisampleState {
            count: sample_count,
            ..Default::default()
        },
        multiview_mask: None,
        cache: None,
    })
}
