//! Render pass drawing retained-UI images as instanced textured quads.

use crate::paint::DrawImage;
use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};
use nalgebra_glm::{Mat4, Vec2, Vec4};
use wgpu::util::DeviceExt;

const INITIAL_INSTANCE_CAPACITY: usize = 256;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ImageVertex {
    corner: Vec2,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct ImageInstance {
    position: Vec2,
    size: Vec2,
    uv_min: Vec2,
    uv_max: Vec2,
    color: Vec4,
    clip_rect: Vec4,
    texture_layer: u32,
    depth: f32,
    _padding0: f32,
    _padding1: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniforms {
    view_projection: Mat4,
    _padding: Vec4,
}

/// Render pass that draws retained-UI images as instanced textured quads.
pub struct PaintImagePass {
    pipeline: wgpu::RenderPipeline,
    instance_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    global_uniform_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    draw_order_buffer: wgpu::Buffer,
    instance_bind_group: wgpu::BindGroup,
    texture_bind_group: Option<wgpu::BindGroup>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    prev_instance_data: Vec<ImageInstance>,
    instance_capacity: usize,
    draw_order_capacity: usize,
    draw_count: u32,
    screen_width: f32,
    screen_height: f32,
}

impl PaintImagePass {
    /// Builds the pass for a color target of `color_format`.
    pub fn new(device: &wgpu::Device, color_format: wgpu::TextureFormat) -> Self {
        let instance_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("DrawImage Instance Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<GlobalUniforms>() as u64,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<ImageInstance>() as u64,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("DrawImage Texture Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("DrawImage Pipeline Layout"),
            bind_group_layouts: &[Some(&instance_layout), Some(&texture_bind_group_layout)],
            immediate_size: 0,
        });

        let shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "DrawImage Shader",
            include_str!("../../shaders/paint_image.wgsl"),
        );

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("DrawImage Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<ImageVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    }],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let vertices = [
            ImageVertex {
                corner: Vec2::new(0.0, 0.0),
            },
            ImageVertex {
                corner: Vec2::new(1.0, 0.0),
            },
            ImageVertex {
                corner: Vec2::new(1.0, 1.0),
            },
            ImageVertex {
                corner: Vec2::new(0.0, 1.0),
            },
        ];
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("DrawImage Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("DrawImage Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let global_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("DrawImage Uniform Buffer"),
            size: std::mem::size_of::<GlobalUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("DrawImage Instance Buffer"),
            size: (std::mem::size_of::<ImageInstance>() * INITIAL_INSTANCE_CAPACITY) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let draw_order_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("DrawImage Draw Order Buffer"),
            size: (std::mem::size_of::<u32>() * INITIAL_INSTANCE_CAPACITY) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instance_bind_group = build_instance_bind_group(
            device,
            &instance_layout,
            &global_uniform_buffer,
            &instance_buffer,
            &draw_order_buffer,
        );

        Self {
            pipeline,
            instance_layout,
            texture_bind_group_layout,
            global_uniform_buffer,
            instance_buffer,
            draw_order_buffer,
            instance_bind_group,
            texture_bind_group: None,
            vertex_buffer,
            index_buffer,
            prev_instance_data: Vec::new(),
            instance_capacity: INITIAL_INSTANCE_CAPACITY,
            draw_order_capacity: INITIAL_INSTANCE_CAPACITY,
            draw_count: 0,
            screen_width: 800.0,
            screen_height: 600.0,
        }
    }

    /// Sets the texture array and sampler the pass samples images from.
    pub fn set_texture_bind_group(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) {
        self.texture_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("DrawImage Texture Bind Group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        }));
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, slot_count: usize, draw_count: usize) {
        let needs_instance = slot_count > self.instance_capacity;
        let needs_order = draw_count > self.draw_order_capacity;
        if !needs_instance && !needs_order {
            return;
        }
        if needs_instance {
            let new_capacity = slot_count.next_power_of_two();
            self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("DrawImage Instance Buffer"),
                size: (std::mem::size_of::<ImageInstance>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instance_capacity = new_capacity;
            self.prev_instance_data.clear();
        }
        if needs_order {
            let new_capacity = draw_count.next_power_of_two();
            self.draw_order_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("DrawImage Draw Order Buffer"),
                size: (std::mem::size_of::<u32>() * new_capacity) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.draw_order_capacity = new_capacity;
        }
        self.instance_bind_group = build_instance_bind_group(
            device,
            &self.instance_layout,
            &self.global_uniform_buffer,
            &self.instance_buffer,
            &self.draw_order_buffer,
        );
    }
}

fn build_instance_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    global: &wgpu::Buffer,
    instances: &wgpu::Buffer,
    draw_order: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("DrawImage Instance Bind Group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: global.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: instances.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: draw_order.as_entire_binding(),
            },
        ],
    })
}

fn unbounded_clip() -> Vec4 {
    Vec4::new(f32::MIN, f32::MIN, f32::MAX, f32::MAX)
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for PaintImagePass {
    fn name(&self) -> &'static str {
        "ui_image_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec![]
    }

    fn writes(&self) -> Vec<&str> {
        vec![]
    }

    fn reads_writes(&self) -> Vec<&str> {
        vec!["color", "depth"]
    }

    fn runs_in_compose_only_phase(&self) -> bool {
        true
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        if let Some((width, height)) = configs.view.viewport_size {
            self.screen_width = width as f32;
            self.screen_height = height as f32;
        }

        let allocator = &configs.ui.render_slots;
        let stable_high_water = allocator.next_image_slot;

        let mut sorted: Vec<&DrawImage> = configs.ui.images.iter().collect();
        sorted.sort_by(|a, b| match a.layer.cmp(&b.layer) {
            std::cmp::Ordering::Equal => a.z_index.cmp(&b.z_index),
            other => other,
        });

        let mut next_frame_slot = stable_high_water;
        let mut entries: Vec<(u32, ImageInstance)> = Vec::with_capacity(sorted.len());
        for image in &sorted {
            let combined_z = (image.layer as i32) * 100000 + image.z_index;
            let depth = 0.1 + (combined_z as f32 / 10000000.0) * 0.8;
            let clip = match image.clip_rect {
                Some(clip) => Vec4::new(clip.min.x, clip.min.y, clip.max.x, clip.max.y),
                None => unbounded_clip(),
            };
            let slot = image
                .entity
                .and_then(|e| allocator.image_slots.get(&e).copied())
                .unwrap_or_else(|| {
                    let s = next_frame_slot;
                    next_frame_slot += 1;
                    s
                });
            entries.push((
                slot,
                ImageInstance {
                    position: image.position,
                    size: image.size,
                    uv_min: image.uv_min,
                    uv_max: image.uv_max,
                    color: image.color,
                    clip_rect: clip,
                    texture_layer: image.texture_index,
                    depth,
                    _padding0: 0.0,
                    _padding1: 0.0,
                },
            ));
        }

        self.draw_count = entries.len() as u32;
        if entries.is_empty() {
            return;
        }

        let view_projection =
            nalgebra_glm::ortho(0.0, self.screen_width, self.screen_height, 0.0, -1.0, 1.0);
        let globals = GlobalUniforms {
            view_projection,
            _padding: Vec4::zeros(),
        };
        queue.write_buffer(
            &self.global_uniform_buffer,
            0,
            bytemuck::cast_slice(&[globals]),
        );

        let total_slot_count = next_frame_slot as usize;
        self.ensure_capacity(device, total_slot_count, entries.len());

        if self.prev_instance_data.len() < total_slot_count {
            self.prev_instance_data
                .resize(total_slot_count, bytemuck::Zeroable::zeroed());
        }

        for (slot, instance) in &entries {
            let slot_idx = *slot as usize;
            if self.prev_instance_data[slot_idx] != *instance {
                queue.write_buffer(
                    &self.instance_buffer,
                    (slot_idx * std::mem::size_of::<ImageInstance>()) as u64,
                    bytemuck::cast_slice(&[*instance]),
                );
                self.prev_instance_data[slot_idx] = *instance;
            }
        }

        let draw_order: Vec<u32> = entries.iter().map(|(slot, _)| *slot).collect();
        queue.write_buffer(
            &self.draw_order_buffer,
            0,
            bytemuck::cast_slice(&draw_order),
        );
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::wgpu::render_configs::RenderInputs>,
    ) -> Result<
        Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>,
        crate::wgpu::rendergraph::RenderGraphError,
    > {
        if !context.is_pass_enabled() || self.draw_count == 0 || self.texture_bind_group.is_none() {
            return Ok(context.into_sub_graph_commands());
        }

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;

        let texture_bind_group = self.texture_bind_group.as_ref().unwrap();

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Image Render Pass"),
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

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.instance_bind_group, &[]);
        render_pass.set_bind_group(1, texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..self.draw_count);

        drop(render_pass);

        Ok(context.into_sub_graph_commands())
    }
}
