use nightshade_wgpu::backend::build;
use nightshade_wgpu::backend::{PassBuild, PassDecl, PassExecutionContext, PassNode, PassPosition};
use nightshade_wgpu::wgpu;
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
struct OutlineMaskUniforms {
    bit_word_count: u32,
    padding: [u32; 3],
}

pub struct OutlineMaskPass {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    cached_bind_group: Option<wgpu::BindGroup>,
    uniform_buffer: wgpu::Buffer,
    outline_bits_buffer: wgpu::Buffer,
    outline_bits_capacity: usize,
    cached_signature: u64,
    cached_word_count: u32,
    enabled: bool,
}

impl OutlineMaskPass {
    pub fn new(build: PassBuild) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let shader = composer.compile_wgsl(
            device,
            "outline_mask.wgsl",
            include_str!("../shaders/outline_mask.wgsl"),
        );

        let uniform_buffer = build::uniform(
            device,
            "Outline Mask Uniform Buffer",
            std::mem::size_of::<OutlineMaskUniforms>() as u64,
        );

        let outline_bits_buffer = build::storage(
            device,
            "Outline Mask Bitmask Buffer",
            (std::mem::size_of::<u32>() * INITIAL_BIT_WORD_CAPACITY) as u64,
        );

        let bind_group_layout = build::bind_layout(
            device,
            "Outline Mask Bind Group Layout",
            &[
                build::texture_at(
                    0,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::TextureViewDimension::D2,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::storage_at(1, false, wgpu::ShaderStages::FRAGMENT),
                build::uniform_at(2, wgpu::ShaderStages::FRAGMENT),
            ],
        );

        let pipeline_layout = build::pipeline_layout(
            device,
            "Outline Mask Pipeline Layout",
            &[&bind_group_layout],
        );

        let pipeline = build::Pipe::new(device, "Outline Mask Pipeline", &shader)
            .layout(&pipeline_layout)
            .target(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::R8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::RED,
            })
            .vertex("vs_main")
            .fragment("fs_main")
            .make();

        Self {
            pipeline,
            bind_group_layout,
            cached_bind_group: None,
            uniform_buffer,
            outline_bits_buffer,
            outline_bits_capacity: INITIAL_BIT_WORD_CAPACITY,
            cached_signature: 0,
            cached_word_count: 0,
            enabled: false,
        }
    }

    fn ensure_bits_capacity(&mut self, device: &wgpu::Device, word_count: usize) {
        if word_count <= self.outline_bits_capacity {
            return;
        }
        let new_capacity = word_count
            .next_power_of_two()
            .max(INITIAL_BIT_WORD_CAPACITY);
        self.outline_bits_buffer = build::storage(
            device,
            "Outline Mask Bitmask Buffer",
            (std::mem::size_of::<u32>() * new_capacity) as u64,
        );
        self.outline_bits_capacity = new_capacity;
        self.cached_bind_group = None;
    }
}

const OUTLINE_MASK_PASS_DECL: PassDecl = PassDecl {
    name: "outline_mask_pass",
    position: PassPosition::PostProcess,
    reads: &["entity_id"],
    writes: &["outline_mask"],
    ..PassDecl::EMPTY
};

impl PassNode<nightshade_wgpu::backend::RenderInputs> for OutlineMaskPass {
    fn decl(&self) -> &PassDecl {
        &OUTLINE_MASK_PASS_DECL
    }

    fn invalidate_bind_groups(&mut self) {
        self.cached_bind_group = None;
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        self.enabled = false;

        let ids: Vec<u32> = configs
            .scene_world
            .query_tag_type::<nightshade_render::components::render_world::Outlined>()
            .map(|entity| entity.id)
            .collect();
        let ids = &ids;

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
        queue.write_buffer(&self.outline_bits_buffer, 0, bytemuck::cast_slice(&bits));

        let uniforms = OutlineMaskUniforms {
            bit_word_count: word_count as u32,
            padding: [0; 3],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        self.cached_signature = signature;
        self.cached_word_count = word_count as u32;
        self.enabled = true;
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, nightshade_wgpu::backend::RenderInputs>,
    ) -> nightshade_wgpu::backend::Result<()> {
        let (mask_view, mask_load_op, mask_store_op) =
            context.get_color_attachment("outline_mask")?;

        if !self.enabled {
            {
                let _render_pass = context
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Outline Mask (Cleared)"),
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
            return Ok(());
        }

        if self.cached_bind_group.is_none() {
            let entity_id_view = context.get_texture_view("entity_id")?;
            self.cached_bind_group = Some(build::binding(
                context.device,
                "Outline Mask Bind Group",
                &self.bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(entity_id_view),
                    self.outline_bits_buffer.as_entire_binding(),
                    self.uniform_buffer.as_entire_binding(),
                ],
            ));
        }

        let mut render_pass = context
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Outline Mask Pass"),
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

        Ok(())
    }
}
