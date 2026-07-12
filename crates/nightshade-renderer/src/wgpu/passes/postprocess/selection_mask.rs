use crate::wgpu::rendergraph::{PassExecutionContext, PassNode};
use std::hash::{Hash, Hasher};

const INITIAL_BIT_WORD_CAPACITY: usize = 64;

fn signature_for_ids(ids: &[u32]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for id in ids {
        id.hash(&mut hasher);
    }
    hasher.finish()
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SelectionMaskUniforms {
    bit_word_count: u32,
    _padding: [u32; 3],
}

pub struct SelectionMaskPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    cached_bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: wgpu::Buffer,
    selection_bits_buffer: wgpu::Buffer,
    selection_bits_capacity: usize,
    cached_signature: u64,
    cached_word_count: u32,
    enabled: bool,
}

impl SelectionMaskPass {
    pub fn new(device: &wgpu::Device, _depth_format: wgpu::TextureFormat) -> Self {
        let shader = crate::wgpu::shader_compose::compile_wgsl(
            device,
            "selection_mask.wgsl",
            include_str!("../../shaders/selection_mask.wgsl"),
        );

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Selection Mask Uniform Buffer"),
            size: std::mem::size_of::<SelectionMaskUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let selection_bits_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Selection Mask Bitmask Buffer"),
            size: (std::mem::size_of::<u32>() * INITIAL_BIT_WORD_CAPACITY) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Selection Mask Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
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
            label: Some("Selection Mask Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Selection Mask Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::RED,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            cached_bind_group: None,
            uniform_buffer,
            selection_bits_buffer,
            selection_bits_capacity: INITIAL_BIT_WORD_CAPACITY,
            cached_signature: 0,
            cached_word_count: 0,
            enabled: false,
        }
    }

    fn ensure_bits_capacity(&mut self, device: &wgpu::Device, word_count: usize) {
        if word_count <= self.selection_bits_capacity {
            return;
        }
        let new_capacity = word_count
            .next_power_of_two()
            .max(INITIAL_BIT_WORD_CAPACITY);
        self.selection_bits_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Selection Mask Bitmask Buffer"),
            size: (std::mem::size_of::<u32>() * new_capacity) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.selection_bits_capacity = new_capacity;
        self.cached_bind_group = None;
    }
}

impl PassNode<crate::wgpu::render_configs::RenderInputs> for SelectionMaskPass {
    fn name(&self) -> &str {
        "selection_mask_pass"
    }

    fn reads(&self) -> Vec<&str> {
        vec!["entity_id"]
    }

    fn writes(&self) -> Vec<&str> {
        vec!["selection_mask"]
    }

    fn invalidate_bind_groups(&mut self) {
        self.cached_bind_group = None;
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &crate::wgpu::render_configs::RenderInputs,
    ) {
        self.enabled = false;

        let ids = &configs.scene.render_selection_outline_ids;

        if ids.is_empty() {
            self.cached_signature = 0;
            self.cached_word_count = 0;
            return;
        }

        let signature = signature_for_ids(ids);
        if self.cached_signature == signature && self.cached_word_count > 0 {
            self.enabled = true;
            return;
        }

        let max_id = ids.iter().copied().max().unwrap_or(0);
        let word_count = ((max_id as usize) / 32) + 1;
        let mut bits = vec![0u32; word_count];
        for id in ids {
            let word_idx = (*id as usize) / 32;
            let bit_idx = *id & 31;
            bits[word_idx] |= 1u32 << bit_idx;
        }

        self.ensure_bits_capacity(device, word_count);
        queue.write_buffer(&self.selection_bits_buffer, 0, bytemuck::cast_slice(&bits));

        let uniforms = SelectionMaskUniforms {
            bit_word_count: word_count as u32,
            _padding: [0; 3],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        self.cached_signature = signature;
        self.cached_word_count = word_count as u32;
        self.enabled = true;
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, crate::wgpu::render_configs::RenderInputs>,
    ) -> crate::wgpu::rendergraph::Result<Vec<crate::wgpu::rendergraph::SubGraphRunCommand<'r>>>
    {
        let (mask_view, mask_load_op, mask_store_op) =
            context.get_color_attachment("selection_mask")?;

        if !self.enabled {
            {
                let _render_pass = context
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Selection Mask (Cleared)"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: mask_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: mask_load_op,
                                store: mask_store_op,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
            }
            return Ok(context.into_sub_graph_commands());
        }

        if self.cached_bind_group.is_none() {
            let entity_id_view = context.get_texture_view("entity_id")?;
            self.cached_bind_group = Some(context.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Selection Mask Bind Group"),
                    layout: &self.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(entity_id_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.selection_bits_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.uniform_buffer.as_entire_binding(),
                        },
                    ],
                },
            ));
        }

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Selection Mask Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: mask_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: mask_load_op,
                        store: mask_store_op,
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

        Ok(context.into_sub_graph_commands())
    }
}
