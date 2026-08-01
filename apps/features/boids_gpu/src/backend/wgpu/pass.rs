use crate::data::{BOIDS_GPU_PASS, CELL_CAPACITY, WORKGROUP};
use crate::queries::{grid_shape, seed_flock};
use crate::resources::FlockSettings;

const CONE_VERTICES: u32 = 18;
use nalgebra_glm::Vec3;
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::RenderInputs;
use nightshade_wgpu::backend::{PassDecl, PassExecutionContext, PassNode, Result};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct StepUniforms {
    step: [f32; 4],
    weights: [f32; 4],
    counts: [u32; 4],
    search: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct DrawUniforms {
    view_projection: [[f32; 4]; 4],
    sun: [f32; 4],
    scale: [f32; 4],
}

struct FlockBuffers {
    positions: nightshade_wgpu::wgpu::Buffer,
    velocities: nightshade_wgpu::wgpu::Buffer,
}

pub struct BoidsGpuPass {
    step_pipelines: [nightshade_wgpu::wgpu::ComputePipeline; 3],
    draw_pipeline: Option<nightshade_wgpu::wgpu::RenderPipeline>,
    draw_shader: nightshade_wgpu::wgpu::ShaderModule,
    draw_pipeline_layout: nightshade_wgpu::wgpu::PipelineLayout,
    step_layout: nightshade_wgpu::wgpu::BindGroupLayout,
    draw_layout: nightshade_wgpu::wgpu::BindGroupLayout,
    draw_target: Option<(nightshade_wgpu::wgpu::TextureFormat, u32)>,

    flock: [FlockBuffers; 2],
    front: usize,
    cell_counts: nightshade_wgpu::wgpu::Buffer,
    cell_items: nightshade_wgpu::wgpu::Buffer,
    step_uniforms: nightshade_wgpu::wgpu::Buffer,
    draw_uniforms: nightshade_wgpu::wgpu::Buffer,

    step_groups: [nightshade_wgpu::wgpu::BindGroup; 2],
    draw_groups: [nightshade_wgpu::wgpu::BindGroup; 2],

    capacity: u32,
    dimension: u32,
    cells: u32,
    settings: FlockSettings,
    stepped_this_frame: bool,
}

impl BoidsGpuPass {
    pub fn new(build: PassBuild<'_>, capacity: u32) -> Self {
        let PassBuild {
            device, composer, ..
        } = build;
        let step_shader = composer.compile_wgsl(
            device,
            "Boids Update Shader",
            include_str!("../../shaders/wgpu/boids_update.wgsl"),
        );
        let draw_shader = composer.compile_wgsl(
            device,
            "Boids Draw Shader",
            include_str!("../../shaders/wgpu/boids_draw.wgsl"),
        );

        let step_layout =
            device.create_bind_group_layout(&nightshade_wgpu::wgpu::BindGroupLayoutDescriptor {
                label: Some("Boids Step Bind Group Layout"),
                entries: &[
                    storage_entry(0, true),
                    storage_entry(1, true),
                    storage_entry(2, false),
                    storage_entry(3, false),
                    storage_entry(4, false),
                    storage_entry(5, false),
                    uniform_entry(6, nightshade_wgpu::wgpu::ShaderStages::COMPUTE),
                ],
            });

        let step_pipeline_layout =
            device.create_pipeline_layout(&nightshade_wgpu::wgpu::PipelineLayoutDescriptor {
                label: Some("Boids Step Pipeline Layout"),
                bind_group_layouts: &[Some(&step_layout)],
                immediate_size: 0,
            });

        let step_pipelines = ["clear_cells", "fill_cells", "steer"].map(|entry| {
            device.create_compute_pipeline(&nightshade_wgpu::wgpu::ComputePipelineDescriptor {
                label: Some("Boids Step Pipeline"),
                layout: Some(&step_pipeline_layout),
                module: &step_shader,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        });

        let draw_layout =
            device.create_bind_group_layout(&nightshade_wgpu::wgpu::BindGroupLayoutDescriptor {
                label: Some("Boids Draw Bind Group Layout"),
                entries: &[
                    vertex_storage_entry(0),
                    vertex_storage_entry(1),
                    uniform_entry(2, nightshade_wgpu::wgpu::ShaderStages::VERTEX_FRAGMENT),
                ],
            });

        let draw_pipeline_layout =
            device.create_pipeline_layout(&nightshade_wgpu::wgpu::PipelineLayoutDescriptor {
                label: Some("Boids Draw Pipeline Layout"),
                bind_group_layouts: &[Some(&draw_layout)],
                immediate_size: 0,
            });

        let settings = FlockSettings::default();
        let (dimension, cells) = grid_shape(settings.bounds, settings.search_radius);

        let flock = [
            FlockBuffers {
                positions: flock_buffer(device, "Boids Positions A", capacity),
                velocities: flock_buffer(device, "Boids Velocities A", capacity),
            },
            FlockBuffers {
                positions: flock_buffer(device, "Boids Positions B", capacity),
                velocities: flock_buffer(device, "Boids Velocities B", capacity),
            },
        ];

        let cell_counts = cell_buffer(device, "Boids Cell Counts", u64::from(cells) * 4);
        let cell_items = cell_buffer(
            device,
            "Boids Cell Items",
            u64::from(cells) * u64::from(CELL_CAPACITY) * 4,
        );
        let step_uniforms = uniform_buffer(
            device,
            "Boids Step Uniforms",
            std::mem::size_of::<StepUniforms>() as u64,
        );
        let draw_uniforms = uniform_buffer(
            device,
            "Boids Draw Uniforms",
            std::mem::size_of::<DrawUniforms>() as u64,
        );

        let step_groups = [0, 1].map(|front| {
            step_bind_group(
                device,
                &step_layout,
                &flock[front],
                &flock[1 - front],
                &cell_counts,
                &cell_items,
                &step_uniforms,
            )
        });
        let draw_groups = [0, 1]
            .map(|front| draw_bind_group(device, &draw_layout, &flock[front], &draw_uniforms));

        Self {
            step_pipelines,
            draw_pipeline: None,
            draw_shader,
            draw_pipeline_layout,
            step_layout,
            draw_layout,
            draw_target: None,
            flock,
            front: 0,
            cell_counts,
            cell_items,
            step_uniforms,
            draw_uniforms,
            step_groups,
            draw_groups,
            capacity,
            dimension,
            cells,
            settings,
            stepped_this_frame: false,
        }
    }

    pub fn hand_over(&mut self, settings: FlockSettings) {
        self.settings = settings;
        self.stepped_this_frame = false;
    }

    pub fn seed(&self, queue: &nightshade_wgpu::wgpu::Queue, count: u32, bounds: f32) {
        let count = count.min(self.capacity);
        let (positions, velocities) = seed_flock(count, bounds);

        for end in &self.flock {
            queue.write_buffer(&end.positions, 0, bytemuck::cast_slice(&positions));
            queue.write_buffer(&end.velocities, 0, bytemuck::cast_slice(&velocities));
        }
    }

    fn resize_grid_if_needed(&mut self, device: &nightshade_wgpu::wgpu::Device) {
        let (dimension, cells) = grid_shape(self.settings.bounds, self.settings.search_radius);
        if dimension == self.dimension {
            return;
        }

        self.dimension = dimension;
        self.cells = cells;
        self.cell_counts = cell_buffer(device, "Boids Cell Counts", u64::from(cells) * 4);
        self.cell_items = cell_buffer(
            device,
            "Boids Cell Items",
            u64::from(cells) * u64::from(CELL_CAPACITY) * 4,
        );

        self.step_groups = [0, 1].map(|front| {
            step_bind_group(
                device,
                &self.step_layout,
                &self.flock[front],
                &self.flock[1 - front],
                &self.cell_counts,
                &self.cell_items,
                &self.step_uniforms,
            )
        });
        self.draw_groups = [0, 1].map(|front| {
            draw_bind_group(
                device,
                &self.draw_layout,
                &self.flock[front],
                &self.draw_uniforms,
            )
        });
    }
}

const BOIDS_GPU_PASS_DECL: PassDecl = PassDecl {
    name: BOIDS_GPU_PASS,
    reads_writes: &["color", "depth"],
    ..PassDecl::EMPTY
};

impl PassNode<RenderInputs> for BoidsGpuPass {
    fn decl(&self) -> &PassDecl {
        &BOIDS_GPU_PASS_DECL
    }

    fn execute<'r, 'e>(
        &mut self,
        mut context: PassExecutionContext<'r, 'e, RenderInputs>,
    ) -> Result<()> {
        let count = self.settings.count.min(self.capacity);
        if count == 0 {
            return Ok(());
        }

        if !self.stepped_this_frame {
            self.stepped_this_frame = true;
            self.resize_grid_if_needed(context.device);
            self.step(&mut context, count);
            self.front = 1 - self.front;
        }

        if !self.settings.visible {
            return Ok(());
        }

        self.draw(&mut context, count)?;
        Ok(())
    }
}

impl BoidsGpuPass {
    fn step(&self, context: &mut PassExecutionContext<'_, '_, RenderInputs>, count: u32) {
        let uniforms = StepUniforms {
            step: [
                self.settings.delta_time.clamp(0.0, 0.05),
                self.settings.bounds,
                self.settings.search_radius,
                self.settings.max_speed,
            ],
            weights: [
                self.settings.separation,
                self.settings.alignment,
                self.settings.cohesion,
                self.settings.turn_strength,
            ],
            counts: [count, self.dimension, self.cells, CELL_CAPACITY],
            search: [
                self.settings.search_radius,
                self.settings.search_radius * self.settings.search_radius,
                0.0,
                0.0,
            ],
        };
        context
            .queue
            .write_buffer(&self.step_uniforms, 0, bytemuck::bytes_of(&uniforms));

        let mut compute_pass =
            context
                .encoder
                .begin_compute_pass(&nightshade_wgpu::wgpu::ComputePassDescriptor {
                    label: Some("Boids Step Pass"),
                    timestamp_writes: None,
                });
        let group = &self.step_groups[self.front];

        let dispatches = [
            self.cells.div_ceil(WORKGROUP),
            count.div_ceil(WORKGROUP),
            count.div_ceil(WORKGROUP),
        ];
        for (pipeline, workgroups) in self.step_pipelines.iter().zip(dispatches) {
            compute_pass.set_pipeline(pipeline);
            compute_pass.set_bind_group(0, group, &[]);
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }
    }

    fn draw(
        &mut self,
        context: &mut PassExecutionContext<'_, '_, RenderInputs>,
        count: u32,
    ) -> Result<()> {
        let color = context.get_texture("color")?;
        let target = (color.format(), color.sample_count());
        if self.draw_target != Some(target) {
            self.draw_pipeline = Some(build_draw_pipeline(
                context.device,
                &self.draw_pipeline_layout,
                &self.draw_shader,
                target,
            ));
            self.draw_target = Some(target);
        }
        let Some(pipeline) = self.draw_pipeline.as_ref() else {
            return Ok(());
        };

        let sun = sun_direction(context.configs);
        let uniforms = DrawUniforms {
            view_projection: context.configs.scene.view_projection,
            sun: [sun.x, sun.y, sun.z, count as f32],
            scale: [1.6, 5.0, 0.0, 0.0],
        };
        context
            .queue
            .write_buffer(&self.draw_uniforms, 0, bytemuck::bytes_of(&uniforms));

        let (color_view, color_load, color_store) = context.get_color_attachment("color")?;
        let (depth_view, depth_load, depth_store) = context.get_depth_attachment("depth")?;

        let mut render_pass =
            context
                .encoder
                .begin_render_pass(&nightshade_wgpu::wgpu::RenderPassDescriptor {
                    label: Some("Boids Draw Pass"),
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
        render_pass.set_bind_group(0, &self.draw_groups[self.front], &[]);
        render_pass.draw(0..CONE_VERTICES, 0..count);
        Ok(())
    }
}

fn build_draw_pipeline(
    device: &nightshade_wgpu::wgpu::Device,
    layout: &nightshade_wgpu::wgpu::PipelineLayout,
    shader: &nightshade_wgpu::wgpu::ShaderModule,
    target: (nightshade_wgpu::wgpu::TextureFormat, u32),
) -> nightshade_wgpu::wgpu::RenderPipeline {
    let (format, sample_count) = target;

    device.create_render_pipeline(&nightshade_wgpu::wgpu::RenderPipelineDescriptor {
        label: Some("Boids Draw Pipeline"),
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
            depth_write_enabled: Some(true),
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

fn flock_buffer(
    device: &nightshade_wgpu::wgpu::Device,
    label: &str,
    capacity: u32,
) -> nightshade_wgpu::wgpu::Buffer {
    device.create_buffer(&nightshade_wgpu::wgpu::BufferDescriptor {
        label: Some(label),
        size: u64::from(capacity) * 16,
        usage: nightshade_wgpu::wgpu::BufferUsages::STORAGE
            | nightshade_wgpu::wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn cell_buffer(
    device: &nightshade_wgpu::wgpu::Device,
    label: &str,
    size: u64,
) -> nightshade_wgpu::wgpu::Buffer {
    device.create_buffer(&nightshade_wgpu::wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: nightshade_wgpu::wgpu::BufferUsages::STORAGE
            | nightshade_wgpu::wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn uniform_buffer(
    device: &nightshade_wgpu::wgpu::Device,
    label: &str,
    size: u64,
) -> nightshade_wgpu::wgpu::Buffer {
    device.create_buffer(&nightshade_wgpu::wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: nightshade_wgpu::wgpu::BufferUsages::UNIFORM
            | nightshade_wgpu::wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn storage_entry(binding: u32, read_only: bool) -> nightshade_wgpu::wgpu::BindGroupLayoutEntry {
    nightshade_wgpu::wgpu::BindGroupLayoutEntry {
        binding,
        visibility: nightshade_wgpu::wgpu::ShaderStages::COMPUTE,
        ty: nightshade_wgpu::wgpu::BindingType::Buffer {
            ty: nightshade_wgpu::wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn vertex_storage_entry(binding: u32) -> nightshade_wgpu::wgpu::BindGroupLayoutEntry {
    nightshade_wgpu::wgpu::BindGroupLayoutEntry {
        binding,
        visibility: nightshade_wgpu::wgpu::ShaderStages::VERTEX,
        ty: nightshade_wgpu::wgpu::BindingType::Buffer {
            ty: nightshade_wgpu::wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn uniform_entry(
    binding: u32,
    visibility: nightshade_wgpu::wgpu::ShaderStages,
) -> nightshade_wgpu::wgpu::BindGroupLayoutEntry {
    nightshade_wgpu::wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: nightshade_wgpu::wgpu::BindingType::Buffer {
            ty: nightshade_wgpu::wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn step_bind_group(
    device: &nightshade_wgpu::wgpu::Device,
    layout: &nightshade_wgpu::wgpu::BindGroupLayout,
    read: &FlockBuffers,
    write: &FlockBuffers,
    cell_counts: &nightshade_wgpu::wgpu::Buffer,
    cell_items: &nightshade_wgpu::wgpu::Buffer,
    uniforms: &nightshade_wgpu::wgpu::Buffer,
) -> nightshade_wgpu::wgpu::BindGroup {
    device.create_bind_group(&nightshade_wgpu::wgpu::BindGroupDescriptor {
        label: Some("Boids Step Bind Group"),
        layout,
        entries: &[
            binding(0, &read.positions),
            binding(1, &read.velocities),
            binding(2, &write.positions),
            binding(3, &write.velocities),
            binding(4, cell_counts),
            binding(5, cell_items),
            binding(6, uniforms),
        ],
    })
}

fn draw_bind_group(
    device: &nightshade_wgpu::wgpu::Device,
    layout: &nightshade_wgpu::wgpu::BindGroupLayout,
    flock: &FlockBuffers,
    uniforms: &nightshade_wgpu::wgpu::Buffer,
) -> nightshade_wgpu::wgpu::BindGroup {
    device.create_bind_group(&nightshade_wgpu::wgpu::BindGroupDescriptor {
        label: Some("Boids Draw Bind Group"),
        layout,
        entries: &[
            binding(0, &flock.positions),
            binding(1, &flock.velocities),
            binding(2, uniforms),
        ],
    })
}

fn binding(
    slot: u32,
    buffer: &nightshade_wgpu::wgpu::Buffer,
) -> nightshade_wgpu::wgpu::BindGroupEntry<'_> {
    nightshade_wgpu::wgpu::BindGroupEntry {
        binding: slot,
        resource: buffer.as_entire_binding(),
    }
}

fn sun_direction(configs: &RenderInputs) -> Vec3 {
    let _ = configs;
    Vec3::new(0.4, 0.8, 0.3)
}
