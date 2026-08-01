use crate::data::{BOIDS_GPU_PASS, CELL_CAPACITY, WORKGROUP};
use crate::queries::{grid_shape, seed_flock};
use crate::resources::FlockSettings;
use nightshade_vulkan::ash::vk;
use nightshade_vulkan::backend::{
    Buffer, Context, MemoryLocation, Pass, PassRecord, PassSetup, create_buffer,
    create_shader_module, destroy_buffer, upload_to_buffer, write_buffer,
};

const BOIDS_PER_TASK: u32 = 32;

const STEP_STAGES: usize = 3;

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
    planes: [[f32; 4]; 6],
}

struct FlockBuffers {
    positions: Buffer,
    velocities: Buffer,
}

pub struct BoidsVulkanPass {
    step_layout: vk::DescriptorSetLayout,
    draw_layout: vk::DescriptorSetLayout,
    step_pipeline_layout: vk::PipelineLayout,
    draw_pipeline_layout: vk::PipelineLayout,
    step_pipelines: [vk::Pipeline; STEP_STAGES],
    draw_pipeline: vk::Pipeline,
    descriptor_pool: vk::DescriptorPool,
    step_sets: Vec<vk::DescriptorSet>,
    draw_sets: Vec<vk::DescriptorSet>,

    flock: [FlockBuffers; 2],
    front: usize,
    frames_in_flight: usize,
    cell_counts: Buffer,
    cell_items: Buffer,
    step_uniforms: Vec<Buffer>,
    draw_uniforms: Vec<Buffer>,

    capacity: u32,
    dimension: u32,
    cells: u32,
    settings: FlockSettings,
}

pub fn create_pass(
    setup: &PassSetup<'_>,
    capacity: u32,
) -> Result<BoidsVulkanPass, Box<dyn std::error::Error>> {
    let context = setup.context;
    let settings = FlockSettings::default();
    let (dimension, cells) = grid_shape(settings.bounds, settings.search_radius);

    let flock = [
        create_flock(context, capacity)?,
        create_flock(context, capacity)?,
    ];
    let cell_counts = create_storage(context, "boid cell counts", u64::from(cells) * 4)?;
    let cell_items = create_storage(
        context,
        "boid cell items",
        u64::from(cells) * u64::from(CELL_CAPACITY) * 4,
    )?;
    let frames_in_flight = setup.frames_in_flight.max(1);
    let mut step_uniforms = Vec::with_capacity(frames_in_flight);
    let mut draw_uniforms = Vec::with_capacity(frames_in_flight);
    for _ in 0..frames_in_flight {
        step_uniforms.push(create_uniform(
            context,
            "boid step uniforms",
            std::mem::size_of::<StepUniforms>() as u64,
        )?);
        draw_uniforms.push(create_uniform(
            context,
            "boid draw uniforms",
            std::mem::size_of::<DrawUniforms>() as u64,
        )?);
    }

    let step_layout = create_step_layout(context)?;
    let draw_layout = create_draw_layout(context)?;
    let step_pipeline_layout = create_pipeline_layout(context, step_layout)?;
    let draw_pipeline_layout = create_pipeline_layout(context, draw_layout)?;
    let step_pipelines = create_step_pipelines(context, step_pipeline_layout)?;
    let draw_pipeline = create_draw_pipeline(setup, draw_pipeline_layout)?;

    let sets_per_layout = 2 * frames_in_flight;
    let descriptor_pool = create_descriptor_pool(context, sets_per_layout)?;
    let step_sets = allocate_sets(context, descriptor_pool, step_layout, sets_per_layout)?;
    let draw_sets = allocate_sets(context, descriptor_pool, draw_layout, sets_per_layout)?;

    let pass = BoidsVulkanPass {
        step_layout,
        draw_layout,
        step_pipeline_layout,
        draw_pipeline_layout,
        step_pipelines,
        draw_pipeline,
        descriptor_pool,
        step_sets,
        draw_sets,
        flock,
        front: 0,
        frames_in_flight,
        cell_counts,
        cell_items,
        step_uniforms,
        draw_uniforms,
        capacity,
        dimension,
        cells,
        settings,
    };
    write_descriptor_sets(context, &pass);
    Ok(pass)
}

pub fn seed(
    pass: &BoidsVulkanPass,
    context: &Context,
    count: u32,
    bounds: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let count = count.min(pass.capacity);
    let (positions, velocities) = seed_flock(count, bounds);
    for end in &pass.flock {
        upload_to_buffer(context, &end.positions, 0, bytemuck::cast_slice(&positions))?;
        upload_to_buffer(
            context,
            &end.velocities,
            0,
            bytemuck::cast_slice(&velocities),
        )?;
    }
    Ok(())
}

pub fn hand_over(pass: &mut BoidsVulkanPass, settings: FlockSettings) {
    pass.settings = settings;
}

impl Pass for BoidsVulkanPass {
    fn name(&self) -> &'static str {
        BOIDS_GPU_PASS
    }

    fn compute(&mut self, record: &mut PassRecord) {
        let count = self.settings.count.min(self.capacity);
        if count == 0 {
            return;
        }

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
        let frame = record.frame_index % self.frames_in_flight;
        write_buffer(&self.step_uniforms[frame], std::slice::from_ref(&uniforms));

        let device = &record.context.device;
        let command_buffer = record.command_buffer;
        let set = self.step_sets[set_index(self.front, frame, self.frames_in_flight)];
        let groups = [
            self.cells.div_ceil(WORKGROUP),
            count.div_ceil(WORKGROUP),
            count.div_ceil(WORKGROUP),
        ];

        unsafe {
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.step_pipeline_layout,
                0,
                &[set],
                &[],
            );
            for (pipeline, group) in self.step_pipelines.iter().zip(groups) {
                device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::COMPUTE, *pipeline);
                device.cmd_dispatch(command_buffer, group.max(1), 1, 1);
                storage_barrier(device, command_buffer);
            }
        }

        self.front = 1 - self.front;
        write_draw_uniforms(self, record, count, frame);
    }

    fn draw(&mut self, record: &mut PassRecord) {
        let count = self.settings.count.min(self.capacity);
        if count == 0 || !self.settings.visible || record.view.is_none() {
            return;
        }

        let device = &record.context.device;
        let command_buffer = record.command_buffer;
        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.draw_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.draw_pipeline_layout,
                0,
                &[self.draw_sets[set_index(
                    self.front,
                    record.frame_index % self.frames_in_flight,
                    self.frames_in_flight,
                )]],
                &[],
            );
        }
        let groups = count.div_ceil(BOIDS_PER_TASK).max(1);
        unsafe {
            record
                .context
                .mesh_shader
                .cmd_draw_mesh_tasks(command_buffer, groups, 1, 1)
        };
    }

    fn destroy(&mut self, context: &Context) {
        let device = &context.device;
        unsafe {
            device.destroy_pipeline(self.draw_pipeline, None);
            for pipeline in self.step_pipelines {
                device.destroy_pipeline(pipeline, None);
            }
            device.destroy_pipeline_layout(self.draw_pipeline_layout, None);
            device.destroy_pipeline_layout(self.step_pipeline_layout, None);
            device.destroy_descriptor_pool(self.descriptor_pool, None);
            device.destroy_descriptor_set_layout(self.draw_layout, None);
            device.destroy_descriptor_set_layout(self.step_layout, None);
        }
        for end in &mut self.flock {
            destroy_buffer(context, &mut end.positions);
            destroy_buffer(context, &mut end.velocities);
        }
        destroy_buffer(context, &mut self.cell_counts);
        destroy_buffer(context, &mut self.cell_items);
        for buffer in &mut self.step_uniforms {
            destroy_buffer(context, buffer);
        }
        for buffer in &mut self.draw_uniforms {
            destroy_buffer(context, buffer);
        }
    }
}

fn set_index(front: usize, frame: usize, frames_in_flight: usize) -> usize {
    front * frames_in_flight + frame
}

fn write_draw_uniforms(pass: &BoidsVulkanPass, record: &PassRecord, count: u32, frame: usize) {
    let Some(view) = record.view else {
        return;
    };
    let mut planes = [[0.0f32; 4]; 6];
    for (slot, plane) in view.frustum_planes.iter().enumerate() {
        planes[slot] = [plane.x, plane.y, plane.z, plane.w];
    }
    let matrix = view.view_projection;
    let mut view_projection = [[0.0f32; 4]; 4];
    for column in 0..4 {
        for row in 0..4 {
            view_projection[column][row] = matrix[(row, column)];
        }
    }
    let half_width = 1.6;
    let half_length = 5.0;
    let uniforms = DrawUniforms {
        view_projection,
        sun: [0.35, 0.85, 0.4, 0.0],
        scale: [half_width, half_length, half_length, count as f32],
        planes,
    };
    write_buffer(&pass.draw_uniforms[frame], std::slice::from_ref(&uniforms));
}

fn storage_barrier(device: &nightshade_vulkan::ash::Device, command_buffer: vk::CommandBuffer) {
    let barrier = vk::MemoryBarrier2::default()
        .src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
        .src_access_mask(vk::AccessFlags2::SHADER_WRITE)
        .dst_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
        .dst_access_mask(vk::AccessFlags2::SHADER_READ | vk::AccessFlags2::SHADER_WRITE);
    let dependency = vk::DependencyInfo::default().memory_barriers(std::slice::from_ref(&barrier));
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
}

fn create_flock(
    context: &Context,
    capacity: u32,
) -> Result<FlockBuffers, Box<dyn std::error::Error>> {
    Ok(FlockBuffers {
        positions: create_storage(context, "boid positions", u64::from(capacity) * 16)?,
        velocities: create_storage(context, "boid velocities", u64::from(capacity) * 16)?,
    })
}

fn create_storage(
    context: &Context,
    name: &str,
    size: u64,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        name,
        size.max(4),
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}

fn create_uniform(
    context: &Context,
    name: &str,
    size: u64,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        name,
        size,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        MemoryLocation::CpuToGpu,
    )
}

fn create_step_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let mut bindings = Vec::new();
    for binding in 0..6u32 {
        bindings.push(
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        );
    }
    bindings.push(
        vk::DescriptorSetLayoutBinding::default()
            .binding(6)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE),
    );
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

fn create_draw_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let mesh_stages = vk::ShaderStageFlags::TASK_EXT | vk::ShaderStageFlags::MESH_EXT;
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(mesh_stages),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::MESH_EXT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(2)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(mesh_stages | vk::ShaderStageFlags::FRAGMENT),
    ];
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

fn create_pipeline_layout(
    context: &Context,
    layout: vk::DescriptorSetLayout,
) -> Result<vk::PipelineLayout, Box<dyn std::error::Error>> {
    let layouts = [layout];
    let create_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
    Ok(unsafe { context.device.create_pipeline_layout(&create_info, None)? })
}

fn create_step_pipelines(
    context: &Context,
    layout: vk::PipelineLayout,
) -> Result<[vk::Pipeline; STEP_STAGES], Box<dyn std::error::Error>> {
    let sources = [
        &include_bytes!("../../shaders/vulkan/boids_clear.comp.spv")[..],
        &include_bytes!("../../shaders/vulkan/boids_fill.comp.spv")[..],
        &include_bytes!("../../shaders/vulkan/boids_steer.comp.spv")[..],
    ];
    let mut pipelines = [vk::Pipeline::null(); STEP_STAGES];
    for (slot, source) in sources.iter().enumerate() {
        let module = create_shader_module(context, source)?;
        let stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(module)
            .name(c"main");
        let create_info = vk::ComputePipelineCreateInfo::default()
            .stage(stage)
            .layout(layout);
        let created = unsafe {
            context
                .device
                .create_compute_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&create_info),
                    None,
                )
                .map_err(|(_, error)| error)?
        };
        pipelines[slot] = created[0];
        unsafe { context.device.destroy_shader_module(module, None) };
    }
    Ok(pipelines)
}

fn create_draw_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let task = create_shader_module(
        context,
        include_bytes!("../../shaders/vulkan/boids.task.spv"),
    )?;
    let mesh = create_shader_module(
        context,
        include_bytes!("../../shaders/vulkan/boids.mesh.spv"),
    )?;
    let fragment = create_shader_module(
        context,
        include_bytes!("../../shaders/vulkan/boids.frag.spv"),
    )?;

    let stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::TASK_EXT)
            .module(task)
            .name(c"main"),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::MESH_EXT)
            .module(mesh)
            .name(c"main"),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment)
            .name(c"main"),
    ];

    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
    let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);
    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::GREATER_OR_EQUAL);
    let blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(false);
    let normal_blend = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::empty());
    let attachments = [blend_attachment, normal_blend, normal_blend];
    let color_blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&attachments);

    let color_formats = [setup.hdr_format, setup.normal_format, setup.motion_format];
    let mut rendering = vk::PipelineRenderingCreateInfo::default()
        .color_attachment_formats(&color_formats)
        .depth_attachment_format(setup.depth_format);

    let create_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .viewport_state(&viewport_state)
        .dynamic_state(&dynamic_state)
        .rasterization_state(&rasterization)
        .multisample_state(&multisample)
        .depth_stencil_state(&depth_stencil)
        .color_blend_state(&color_blend)
        .layout(layout)
        .push_next(&mut rendering);

    let created = unsafe {
        context
            .device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&create_info),
                None,
            )
            .map_err(|(_, error)| error)?
    };

    unsafe {
        context.device.destroy_shader_module(task, None);
        context.device.destroy_shader_module(mesh, None);
        context.device.destroy_shader_module(fragment, None);
    }
    Ok(created[0])
}

fn create_descriptor_pool(
    context: &Context,
    sets_per_layout: usize,
) -> Result<vk::DescriptorPool, Box<dyn std::error::Error>> {
    let sets = (sets_per_layout * 2) as u32;
    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(sets * 6),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(sets),
    ];
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&sizes)
        .max_sets(sets);
    Ok(unsafe { context.device.create_descriptor_pool(&create_info, None)? })
}

fn allocate_sets(
    context: &Context,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    count: usize,
) -> Result<Vec<vk::DescriptorSet>, Box<dyn std::error::Error>> {
    let layouts = vec![layout; count];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    Ok(unsafe { context.device.allocate_descriptor_sets(&allocate_info)? })
}

fn write_descriptor_sets(context: &Context, pass: &BoidsVulkanPass) {
    for front in 0..2usize {
        let back = 1 - front;
        let step_buffers = [
            pass.flock[front].positions.buffer,
            pass.flock[front].velocities.buffer,
            pass.flock[back].positions.buffer,
            pass.flock[back].velocities.buffer,
            pass.cell_counts.buffer,
            pass.cell_items.buffer,
        ];
        let infos = step_buffers
            .iter()
            .map(|buffer| {
                vk::DescriptorBufferInfo::default()
                    .buffer(*buffer)
                    .offset(0)
                    .range(vk::WHOLE_SIZE)
            })
            .collect::<Vec<_>>();

        let draw_positions = vk::DescriptorBufferInfo::default()
            .buffer(pass.flock[front].positions.buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);
        let draw_velocities = vk::DescriptorBufferInfo::default()
            .buffer(pass.flock[front].velocities.buffer)
            .offset(0)
            .range(vk::WHOLE_SIZE);

        for frame in 0..pass.frames_in_flight {
            let slot = set_index(front, frame, pass.frames_in_flight);

            let step_uniform = vk::DescriptorBufferInfo::default()
                .buffer(pass.step_uniforms[frame].buffer)
                .offset(0)
                .range(vk::WHOLE_SIZE);
            let mut writes = infos
                .iter()
                .enumerate()
                .map(|(binding, info)| {
                    vk::WriteDescriptorSet::default()
                        .dst_set(pass.step_sets[slot])
                        .dst_binding(binding as u32)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .buffer_info(std::slice::from_ref(info))
                })
                .collect::<Vec<_>>();
            writes.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(pass.step_sets[slot])
                    .dst_binding(6)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(std::slice::from_ref(&step_uniform)),
            );
            unsafe { context.device.update_descriptor_sets(&writes, &[]) };

            let draw_uniform = vk::DescriptorBufferInfo::default()
                .buffer(pass.draw_uniforms[frame].buffer)
                .offset(0)
                .range(vk::WHOLE_SIZE);
            let draw_writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(pass.draw_sets[slot])
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(std::slice::from_ref(&draw_positions)),
                vk::WriteDescriptorSet::default()
                    .dst_set(pass.draw_sets[slot])
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(std::slice::from_ref(&draw_velocities)),
                vk::WriteDescriptorSet::default()
                    .dst_set(pass.draw_sets[slot])
                    .dst_binding(2)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(std::slice::from_ref(&draw_uniform)),
            ];
            unsafe { context.device.update_descriptor_sets(&draw_writes, &[]) };
        }
    }
}
