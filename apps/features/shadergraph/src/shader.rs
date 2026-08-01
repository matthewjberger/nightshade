use crate::resources::{Plucked, SLOTS};
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, Result};

pub const GRAPH_PASS: &str = "shadergraph_graph_pass";

const FRAME: &str = include_str!("shaders/frame.wgsl");
const OPENING: &str = "@fragment\nfn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n    let uv = vec2<f32>(in.uv.x * u.aspect, in.uv.y);";
const CLOSING: &str = "}";
const EMPTY_BODY: &str = "    return vec4<f32>(0.03, 0.03, 0.04, 1.0);";

pub fn whole_shader(body: &str) -> String {
    format!("{FRAME}\n{OPENING}\n{body}\n{CLOSING}\n")
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GraphUniforms {
    time: f32,
    aspect: f32,
    width: f32,
    height: f32,
}

pub struct GraphPass {
    layout: nightshade_wgpu::wgpu::PipelineLayout,
    uniforms: nightshade_wgpu::wgpu::Buffer,
    bind_group: nightshade_wgpu::wgpu::BindGroup,
    picture_layout: nightshade_wgpu::wgpu::BindGroupLayout,
    picture_sampler: nightshade_wgpu::wgpu::Sampler,
    picture_views: Vec<nightshade_wgpu::wgpu::TextureView>,
    picture_group: Option<nightshade_wgpu::wgpu::BindGroup>,
    wanted_pictures: Vec<Option<Plucked>>,
    picture_revision: u64,
    pictures_built: u64,
    pipeline: Option<nightshade_wgpu::wgpu::RenderPipeline>,
    target: Option<(nightshade_wgpu::wgpu::TextureFormat, u32)>,
    body: String,
    revision: u64,
    built: u64,
    inset: [f32; 4],
    pub time: f32,
}

impl GraphPass {
    pub fn new(build: nightshade_wgpu::backend::PassBuild<'_>) -> Self {
        let nightshade_wgpu::backend::PassBuild { device, .. } = build;
        let bind_layout =
            device.create_bind_group_layout(&nightshade_wgpu::wgpu::BindGroupLayoutDescriptor {
                label: Some("Shader Graph Bind Layout"),
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
        let mut picture_entries = vec![nightshade_wgpu::wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: nightshade_wgpu::wgpu::ShaderStages::FRAGMENT,
            ty: nightshade_wgpu::wgpu::BindingType::Sampler(
                nightshade_wgpu::wgpu::SamplerBindingType::Filtering,
            ),
            count: None,
        }];
        for slot in 0..SLOTS {
            picture_entries.push(nightshade_wgpu::wgpu::BindGroupLayoutEntry {
                binding: slot as u32 + 1,
                visibility: nightshade_wgpu::wgpu::ShaderStages::FRAGMENT,
                ty: nightshade_wgpu::wgpu::BindingType::Texture {
                    sample_type: nightshade_wgpu::wgpu::TextureSampleType::Float {
                        filterable: true,
                    },
                    view_dimension: nightshade_wgpu::wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            });
        }
        let picture_layout =
            device.create_bind_group_layout(&nightshade_wgpu::wgpu::BindGroupLayoutDescriptor {
                label: Some("Shader Graph Picture Layout"),
                entries: &picture_entries,
            });
        let picture_sampler = device.create_sampler(&nightshade_wgpu::wgpu::SamplerDescriptor {
            label: Some("Shader Graph Picture Sampler"),
            address_mode_u: nightshade_wgpu::wgpu::AddressMode::Repeat,
            address_mode_v: nightshade_wgpu::wgpu::AddressMode::Repeat,
            address_mode_w: nightshade_wgpu::wgpu::AddressMode::Repeat,
            mag_filter: nightshade_wgpu::wgpu::FilterMode::Linear,
            min_filter: nightshade_wgpu::wgpu::FilterMode::Linear,
            mipmap_filter: nightshade_wgpu::wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let layout =
            device.create_pipeline_layout(&nightshade_wgpu::wgpu::PipelineLayoutDescriptor {
                label: Some("Shader Graph Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_layout), Some(&picture_layout)],
                immediate_size: 0,
            });
        let uniforms = device.create_buffer(&nightshade_wgpu::wgpu::BufferDescriptor {
            label: Some("Shader Graph Uniforms"),
            size: std::mem::size_of::<GraphUniforms>() as u64,
            usage: nightshade_wgpu::wgpu::BufferUsages::UNIFORM
                | nightshade_wgpu::wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&nightshade_wgpu::wgpu::BindGroupDescriptor {
            label: Some("Shader Graph Bind Group"),
            layout: &bind_layout,
            entries: &[nightshade_wgpu::wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });
        Self {
            layout,
            uniforms,
            bind_group,
            picture_layout,
            picture_sampler,
            picture_views: Vec::new(),
            picture_group: None,
            wanted_pictures: vec![None; SLOTS],
            picture_revision: 0,
            pictures_built: u64::MAX,
            pipeline: None,
            target: None,
            body: EMPTY_BODY.to_string(),
            revision: 0,
            built: 0,
            inset: [0.0; 4],
            time: 0.0,
        }
    }

    pub fn take_the_inset(&mut self, left: f32, right: f32, top: f32, bottom: f32) {
        self.inset = [left, right, top, bottom];
    }

    pub fn take_the_pictures(&mut self, held: &[Option<Plucked>], revision: u64) {
        if self.picture_revision == revision {
            return;
        }
        self.wanted_pictures = held.to_vec();
        self.picture_revision = revision;
    }

    fn build_the_pictures(
        &mut self,
        device: &nightshade_wgpu::wgpu::Device,
        queue: &nightshade_wgpu::wgpu::Queue,
    ) {
        let mut views = Vec::with_capacity(SLOTS);
        for slot in 0..SLOTS {
            let held = self
                .wanted_pictures
                .get(slot)
                .and_then(|held| held.as_ref());
            let (width, height, rgba) = match held {
                Some(picture) if picture.width > 0 && picture.height > 0 => {
                    (picture.width, picture.height, picture.rgba.clone())
                }
                _ => (1, 1, vec![255, 255, 255, 255]),
            };
            let texture = device.create_texture(&nightshade_wgpu::wgpu::TextureDescriptor {
                label: Some("Shader Graph Picture"),
                size: nightshade_wgpu::wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: nightshade_wgpu::wgpu::TextureDimension::D2,
                format: nightshade_wgpu::wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: nightshade_wgpu::wgpu::TextureUsages::TEXTURE_BINDING
                    | nightshade_wgpu::wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                nightshade_wgpu::wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: nightshade_wgpu::wgpu::Origin3d::ZERO,
                    aspect: nightshade_wgpu::wgpu::TextureAspect::All,
                },
                &rgba,
                nightshade_wgpu::wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                nightshade_wgpu::wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            views.push(
                texture.create_view(&nightshade_wgpu::wgpu::TextureViewDescriptor::default()),
            );
        }

        let mut entries = vec![nightshade_wgpu::wgpu::BindGroupEntry {
            binding: 0,
            resource: nightshade_wgpu::wgpu::BindingResource::Sampler(&self.picture_sampler),
        }];
        for (slot, view) in views.iter().enumerate() {
            entries.push(nightshade_wgpu::wgpu::BindGroupEntry {
                binding: slot as u32 + 1,
                resource: nightshade_wgpu::wgpu::BindingResource::TextureView(view),
            });
        }
        self.picture_group = Some(device.create_bind_group(
            &nightshade_wgpu::wgpu::BindGroupDescriptor {
                label: Some("Shader Graph Picture Group"),
                layout: &self.picture_layout,
                entries: &entries,
            },
        ));
        self.picture_views = views;
        self.pictures_built = self.picture_revision;
    }

    pub fn take_the_body(&mut self, body: String, revision: u64) {
        if self.revision == revision {
            return;
        }
        self.body = if body.trim().is_empty() {
            EMPTY_BODY.to_string()
        } else {
            body
        };
        self.revision = revision;
    }
}

const GRAPH_PASS_DECL: PassDecl = PassDecl {
    name: GRAPH_PASS,
    reads_writes: &["color", "depth"],
    ..PassDecl::EMPTY
};

impl PassNode<RenderInputs> for GraphPass {
    fn decl(&self) -> &PassDecl {
        &GRAPH_PASS_DECL
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> Result<()> {
        if self.pictures_built != self.picture_revision {
            self.build_the_pictures(context.device, context.queue);
        }
        let Some(picture_group) = self.picture_group.as_ref() else {
            return Ok(());
        };

        let color = context.get_texture("color")?;
        let target = (color.format(), color.sample_count());
        if self.target != Some(target) || self.built != self.revision {
            let source = whole_shader(&self.body);
            let module = context.device.create_shader_module(
                nightshade_wgpu::wgpu::ShaderModuleDescriptor {
                    label: Some("Shader Graph Shader"),
                    source: nightshade_wgpu::wgpu::ShaderSource::Wgsl(source.into()),
                },
            );
            self.pipeline = Some(build_pipeline(
                context.device,
                &self.layout,
                &module,
                target,
            ));
            self.target = Some(target);
            self.built = self.revision;
        }
        let Some(pipeline) = self.pipeline.as_ref() else {
            return Ok(());
        };

        let (width, height) = context.get_texture_size("color")?;
        let [left, right, top, bottom] = self.inset;
        let near = (left.max(0.0)).min(width as f32);
        let high = (top.max(0.0)).min(height as f32);
        let span = (width as f32 - near - right.max(0.0)).floor();
        let drop = (height as f32 - high - bottom.max(0.0)).floor();
        if span < 1.0 || drop < 1.0 {
            return Ok(());
        }
        let uniforms = GraphUniforms {
            time: self.time,
            aspect: span / drop,
            width: span,
            height: drop,
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
                    label: Some("Shader Graph Pass"),
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
        render_pass.set_viewport(near.floor(), high.floor(), span, drop, 0.0, 1.0);
        render_pass.set_scissor_rect(near as u32, high as u32, span as u32, drop as u32);
        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, picture_group, &[]);
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
        label: Some("Shader Graph Pipeline"),
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
            depth_compare: Some(nightshade_wgpu::wgpu::CompareFunction::GreaterEqual),
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
