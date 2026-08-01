use nalgebra_glm::{Mat4, Vec2, Vec4};
use nightshade_paint::DrawImage;
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, PassPosition, PassStage};
use nightshade_wgpu::wgpu;
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
    rotation: f32,
    flags: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniforms {
    view_projection: Mat4,
    padding: Vec4,
}

pub struct PaintImagePass {
    pipeline: wgpu::RenderPipeline,
    instance_layout: wgpu::BindGroupLayout,
    global_uniform_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    draw_order_buffer: wgpu::Buffer,
    instance_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    prev_instance_data: Vec<ImageInstance>,
    instance_capacity: usize,
    draw_order_capacity: usize,
    draw_count: u32,
    screen_width: f32,
    screen_height: f32,
    draw_list: nightshade_paint::PaintDrawList,
    texture_pool: nightshade_wgpu::backend::texture_array_pool::TextureArrayPool,
}

impl PaintImagePass {
    pub fn new(build: PassBuild, color_format: wgpu::TextureFormat) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let instance_layout = build::bind_layout(
            device,
            "DrawImage Instance Layout",
            &[
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
                build::storage_at(2, false, wgpu::ShaderStages::VERTEX),
            ],
        );

        let texture_bind_group_layout = build::bind_layout(
            device,
            "DrawImage Texture Layout",
            &[
                build::texture_at(
                    0,
                    wgpu::TextureSampleType::Float { filterable: true },
                    wgpu::TextureViewDimension::D2Array,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::filtering_at(1),
                build::filtering_at(2),
            ],
        );

        let pipeline_layout = build::pipeline_layout(
            device,
            "DrawImage Pipeline Layout",
            &[&instance_layout, &texture_bind_group_layout],
        );

        let shader = composer.compile_wgsl(
            device,
            "DrawImage Shader",
            include_str!("../../shaders/paint_image.wgsl"),
        );

        let pipeline = build::Pipe::new(device, "DrawImage Pipeline", &shader)
            .layout(&pipeline_layout)
            .buffers(&[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<ImageVertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                }],
            }])
            .target(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })
            .depth(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
            .vertex("vs_main")
            .fragment("fs_main")
            .make();

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

        let global_uniform_buffer = build::uniform(
            device,
            "DrawImage Uniform Buffer",
            std::mem::size_of::<GlobalUniforms>() as u64,
        );

        let instance_buffer = build::storage(
            device,
            "DrawImage Instance Buffer",
            (std::mem::size_of::<ImageInstance>() * INITIAL_INSTANCE_CAPACITY) as u64,
        );

        let draw_order_buffer = build::storage(
            device,
            "DrawImage Draw Order Buffer",
            (std::mem::size_of::<u32>() * INITIAL_INSTANCE_CAPACITY) as u64,
        );

        let instance_bind_group = build_instance_bind_group(
            device,
            &instance_layout,
            &global_uniform_buffer,
            &instance_buffer,
            &draw_order_buffer,
        );

        let texture_pool = nightshade_wgpu::backend::texture_array_pool::create_texture_array_pool(
            device,
            nightshade_paint::TEXTURE_LAYER_SIZE,
            nightshade_paint::TEXTURE_MAX_LAYERS,
        );
        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("DrawImage Nearest Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let texture_bind_group = build::binding(
            device,
            "DrawImage Texture Bind Group",
            &texture_bind_group_layout,
            vec![
                wgpu::BindingResource::TextureView(&texture_pool.view),
                wgpu::BindingResource::Sampler(&texture_pool.sampler),
                wgpu::BindingResource::Sampler(&nearest_sampler),
            ],
        );

        Self {
            pipeline,
            instance_layout,
            global_uniform_buffer,
            instance_buffer,
            draw_order_buffer,
            instance_bind_group,
            texture_bind_group,
            vertex_buffer,
            index_buffer,
            prev_instance_data: Vec::new(),
            instance_capacity: INITIAL_INSTANCE_CAPACITY,
            draw_order_capacity: INITIAL_INSTANCE_CAPACITY,
            draw_count: 0,
            screen_width: 800.0,
            screen_height: 600.0,
            draw_list: nightshade_paint::PaintDrawList::default(),
            texture_pool,
        }
    }

    pub fn upload_image_layer(
        &self,
        queue: &wgpu::Queue,
        upload: &nightshade_paint::ImageLayerUpload,
    ) {
        nightshade_wgpu::backend::texture_array_pool::upload_layer(
            &self.texture_pool,
            queue,
            upload.layer,
            &upload.rgba_data,
            upload.width,
            upload.height,
        );
    }

    pub fn swap_draw_list(
        &mut self,
        draw_list: nightshade_paint::PaintDrawList,
    ) -> nightshade_paint::PaintDrawList {
        std::mem::replace(&mut self.draw_list, draw_list)
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, slot_count: usize, draw_count: usize) {
        let needs_instance = slot_count > self.instance_capacity;
        let needs_order = draw_count > self.draw_order_capacity;
        if !needs_instance && !needs_order {
            return;
        }
        if needs_instance {
            let new_capacity = slot_count.next_power_of_two();
            self.instance_buffer = build::storage(
                device,
                "DrawImage Instance Buffer",
                (std::mem::size_of::<ImageInstance>() * new_capacity) as u64,
            );
            self.instance_capacity = new_capacity;
            self.prev_instance_data.clear();
        }
        if needs_order {
            let new_capacity = draw_count.next_power_of_two();
            self.draw_order_buffer = build::storage(
                device,
                "DrawImage Draw Order Buffer",
                (std::mem::size_of::<u32>() * new_capacity) as u64,
            );
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
    build::binding(
        device,
        "DrawImage Instance Bind Group",
        layout,
        vec![
            global.as_entire_binding(),
            instances.as_entire_binding(),
            draw_order.as_entire_binding(),
        ],
    )
}

fn unbounded_clip() -> Vec4 {
    Vec4::new(f32::MIN, f32::MIN, f32::MAX, f32::MAX)
}

const PAINT_IMAGE_PASS_DECL: PassDecl = PassDecl {
    name: super::PAINT_IMAGE_PASS,
    reads_writes: &["color", "depth"],
    stage: PassStage::Swapchain,
    position: PassPosition::Swapchain,
    compose_only_phase: true,
    ..PassDecl::EMPTY
};

impl PassNode<nightshade_wgpu::backend::RenderInputs> for PaintImagePass {
    fn decl(&self) -> &PassDecl {
        &PAINT_IMAGE_PASS_DECL
    }

    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        configs: &nightshade_wgpu::backend::RenderInputs,
    ) {
        if let Some((width, height)) = configs.view.viewport_size {
            self.screen_width = width as f32;
            self.screen_height = height as f32;
        }

        let draw_list = std::mem::take(&mut self.draw_list);
        let allocator = &draw_list.render_slots;
        let stable_high_water = allocator.next_image_slot;

        let mut sorted: Vec<&DrawImage> = draw_list.images.iter().collect();
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
                    rotation: image.rotation,
                    flags: if image.nearest {
                        nightshade_paint::IMAGE_FLAG_NEAREST
                    } else {
                        0
                    },
                },
            ));
        }

        self.draw_count = entries.len() as u32;
        if entries.is_empty() {
            self.draw_list = draw_list;
            return;
        }

        let view_projection =
            nalgebra_glm::ortho(0.0, self.screen_width, self.screen_height, 0.0, -1.0, 1.0);
        let globals = GlobalUniforms {
            view_projection,
            padding: Vec4::zeros(),
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

        self.draw_list = draw_list;
    }

    fn execute<'r, 'e>(
        &mut self,
        context: PassExecutionContext<'r, 'e, nightshade_wgpu::backend::RenderInputs>,
    ) -> Result<(), nightshade_wgpu::backend::RenderGraphError> {
        if !context.is_pass_enabled() || self.draw_count == 0 {
            return Ok(());
        }

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;

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
        render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..self.draw_count);

        drop(render_pass);

        Ok(())
    }
}
