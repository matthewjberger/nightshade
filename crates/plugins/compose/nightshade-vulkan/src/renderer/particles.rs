use ash::vk;
use gpu_allocator::MemoryLocation;

use crate::renderer::context::Context;
use crate::renderer::image_table::{
    ImageTable, ImageUpload, create_image_table, destroy_image_table, upload_image,
};
use crate::renderer::memory::{Buffer, create_buffer, destroy_buffer, write_buffer};
use crate::renderer::pass::{Pass, PassRecord, PassSetup, create_shader_module};

pub const PARTICLES_PASS: &str = "particles";

pub const MAX_PARTICLES: u32 = 1_000_000;

fn particle_budget(context: &Context) -> u32 {
    let properties = unsafe {
        context
            .instance
            .get_physical_device_properties(context.physical_device)
    };
    let stride = std::mem::size_of::<[[f32; 4]; 8]>() as u64;
    let room =
        (properties.limits.max_storage_buffer_range as u64 / stride).min(u32::MAX as u64) as u32;
    MAX_PARTICLES.min(room)
}
pub const MAX_EMITTERS: u32 = 512;
const MAX_PARTICLE_TEXTURES: u32 = 64;

const SPAWN_GROUP_SIZE: u32 = 256;
const UPDATE_GROUP_SIZE: u32 = 256;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuEmitter {
    pub position: [f32; 4],
    pub direction: [f32; 4],
    pub velocity_range: [f32; 4],
    pub lifetime_range: [f32; 4],
    pub size_range: [f32; 4],
    pub gravity: [f32; 4],
    pub color_gradient: [[f32; 4]; 16],
    pub gradient_count: u32,
    pub spawn_count: u32,
    pub emitter_id: u32,
    pub shape_type: u32,
    pub shape_params: [f32; 4],
    pub turbulence: [f32; 4],
    pub emissive_strength: f32,
    pub drag: f32,
    pub emitter_type: u32,
    pub texture_index: u32,
    pub size_curve: [[f32; 4]; 8],
    pub size_curve_count: u32,
    pub size_curve_padding: [u32; 3],
    pub opacity_curve: [[f32; 4]; 8],
    pub opacity_curve_count: u32,
    pub opacity_curve_padding: [u32; 3],
}

impl Default for GpuEmitter {
    fn default() -> Self {
        Self {
            position: [0.0; 4],
            direction: [0.0, 1.0, 0.0, 0.0],
            velocity_range: [1.0, 3.0, 0.3, 0.0],
            lifetime_range: [1.0, 2.0, 0.0, 0.0],
            size_range: [0.1, 0.0, 0.0, 0.0],
            gravity: [0.0, -9.81, 0.0, 0.0],
            color_gradient: [[0.0; 4]; 16],
            gradient_count: 0,
            spawn_count: 0,
            emitter_id: 0,
            shape_type: 0,
            shape_params: [0.0; 4],
            turbulence: [0.0; 4],
            emissive_strength: 1.0,
            drag: 0.0,
            emitter_type: 0,
            texture_index: 0,
            size_curve: [[0.0; 4]; 8],
            size_curve_count: 0,
            size_curve_padding: [0; 3],
            opacity_curve: [[0.0; 4]; 8],
            opacity_curve_count: 0,
            opacity_curve_padding: [0; 3],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Simulation {
    delta_time: f32,
    time: f32,
    max_particles: u32,
    padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Camera {
    view_projection: [[f32; 4]; 4],
    camera_right: [f32; 4],
    camera_up: [f32; 4],
}

pub struct PendingTexture {
    pub slot: u32,
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Default)]
pub struct ParticleFrameData {
    pub emitters: Vec<GpuEmitter>,
    pub delta_time: f32,
    pub time: f32,
    pub view_projection: [[f32; 4]; 4],
    pub camera_right: [f32; 4],
    pub camera_up: [f32; 4],
    pub textures: Vec<PendingTexture>,
}

pub struct ParticlesPass {
    reset_pipeline: vk::Pipeline,
    spawn_pipeline: vk::Pipeline,
    update_pipeline: vk::Pipeline,
    compute_layout: vk::PipelineLayout,
    compute_descriptor_layout: vk::DescriptorSetLayout,
    compute_set: vk::DescriptorSet,
    draw_pipeline: vk::Pipeline,
    draw_layout: vk::PipelineLayout,
    draw_descriptor_layout: vk::DescriptorSetLayout,
    draw_set: vk::DescriptorSet,
    pool: vk::DescriptorPool,
    textures: ImageTable,
    particles: Buffer,
    emitters: Buffer,
    simulation: Buffer,
    free_indices: Buffer,
    free_count: Buffer,
    alive_indices: Buffer,
    alive_count: Buffer,
    draw_command: Buffer,
    camera: Buffer,
    data: ParticleFrameData,
    started: bool,
    budget: u32,
}

pub fn create_pass(setup: &PassSetup<'_>) -> Result<ParticlesPass, Box<dyn std::error::Error>> {
    let context = setup.context;
    let textures = create_image_table(
        context,
        MAX_PARTICLE_TEXTURES,
        vk::ShaderStageFlags::FRAGMENT,
    )?;

    let budget = particle_budget(context);
    let particles = create_storage(
        context,
        "particles",
        budget as u64 * std::mem::size_of::<[[f32; 4]; 8]>() as u64,
    )?;
    let emitters = create_shared(
        context,
        "particle emitters",
        MAX_EMITTERS as u64 * std::mem::size_of::<GpuEmitter>() as u64,
    )?;
    let simulation = create_buffer(
        context,
        "particle simulation",
        std::mem::size_of::<Simulation>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;
    let free_indices = create_shared(
        context,
        "particle free list",
        budget as u64 * std::mem::size_of::<u32>() as u64,
    )?;
    let free_count = create_shared(
        context,
        "particle free count",
        std::mem::size_of::<u32>() as u64,
    )?;
    let alive_indices = create_storage(
        context,
        "particle alive list",
        budget as u64 * std::mem::size_of::<u32>() as u64,
    )?;
    let alive_count = create_storage(
        context,
        "particle alive count",
        std::mem::size_of::<u32>() as u64,
    )?;
    let draw_command = create_buffer(
        context,
        "particle draw command",
        std::mem::size_of::<[u32; 4]>() as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::INDIRECT_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;
    let camera = create_buffer(
        context,
        "particle camera",
        std::mem::size_of::<Camera>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;

    let compute_descriptor_layout = create_compute_layout(context)?;
    let draw_descriptor_layout = create_draw_layout(context)?;
    let pool = create_descriptor_pool(context)?;
    let compute_set = allocate_set(context, pool, compute_descriptor_layout)?;
    let draw_set = allocate_set(context, pool, draw_descriptor_layout)?;

    let compute_layout = create_pipeline_layout(context, &[compute_descriptor_layout])?;
    let draw_layout = create_pipeline_layout(context, &[draw_descriptor_layout, textures.layout])?;

    let reset_pipeline = create_compute_pipeline(
        context,
        compute_layout,
        include_bytes!("shaders/particle_reset.comp.spv"),
    )?;
    let spawn_pipeline = create_compute_pipeline(
        context,
        compute_layout,
        include_bytes!("shaders/particle_spawn.comp.spv"),
    )?;
    let update_pipeline = create_compute_pipeline(
        context,
        compute_layout,
        include_bytes!("shaders/particle_update.comp.spv"),
    )?;
    let draw_pipeline = create_draw_pipeline(setup, draw_layout)?;

    let pass = ParticlesPass {
        budget,
        reset_pipeline,
        spawn_pipeline,
        update_pipeline,
        compute_layout,
        compute_descriptor_layout,
        compute_set,
        draw_pipeline,
        draw_layout,
        draw_descriptor_layout,
        draw_set,
        pool,
        textures,
        particles,
        emitters,
        simulation,
        free_indices,
        free_count,
        alive_indices,
        alive_count,
        draw_command,
        camera,
        data: ParticleFrameData::default(),
        started: false,
    };
    write_sets(context, &pass);
    fill_free_list(&pass);
    Ok(pass)
}

pub fn hand_over(pass: &mut ParticlesPass, data: ParticleFrameData) {
    pass.data = data;
}

impl Pass for ParticlesPass {
    fn name(&self) -> &'static str {
        PARTICLES_PASS
    }

    fn compute(&mut self, record: &mut PassRecord) {
        let context = record.context;
        for texture in std::mem::take(&mut self.data.textures) {
            if texture.slot == 0 {
                continue;
            }
            if let Err(error) = upload_image(
                &mut self.textures,
                context,
                texture.slot - 1,
                ImageUpload {
                    width: texture.width,
                    height: texture.height,
                    pixels: &texture.rgba,
                    srgb: true,
                    mips: true,
                    format: nightshade_render::data::decoded_image::TexelFormat::Rgba8,
                    levels: &[],
                },
            ) {
                tracing::error!("a particle texture was not uploaded: {error}");
            }
        }

        write_buffer(
            &self.simulation,
            std::slice::from_ref(&Simulation {
                delta_time: self.data.delta_time,
                time: self.data.time,
                max_particles: self.budget,
                padding: 0,
            }),
        );
        write_buffer(
            &self.camera,
            std::slice::from_ref(&Camera {
                view_projection: self.data.view_projection,
                camera_right: self.data.camera_right,
                camera_up: self.data.camera_up,
            }),
        );
        let count = self.data.emitters.len().min(MAX_EMITTERS as usize);
        let emitters = &self.data.emitters[..count];
        if !emitters.is_empty() {
            write_buffer(&self.emitters, emitters);
        }
        write_buffer(&self.draw_command, &[6u32, 0, 0, 0]);

        let device = &context.device;
        unsafe {
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.compute_layout,
                0,
                &[self.compute_set],
                &[],
            );
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.reset_pipeline,
            );
            device.cmd_dispatch(record.command_buffer, 1, 1, 1);
            barrier(device, record.command_buffer);

            if !emitters.is_empty() {
                device.cmd_bind_pipeline(
                    record.command_buffer,
                    vk::PipelineBindPoint::COMPUTE,
                    self.spawn_pipeline,
                );
                device.cmd_dispatch(record.command_buffer, emitters.len() as u32, 1, 1);
                barrier(device, record.command_buffer);
            }

            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.update_pipeline,
            );
            device.cmd_dispatch(
                record.command_buffer,
                self.budget.div_ceil(UPDATE_GROUP_SIZE),
                1,
                1,
            );
            draw_barrier(device, record.command_buffer);
        }
        self.started = true;
    }

    fn draw(&mut self, record: &mut PassRecord) {
        if !self.started {
            return;
        }
        let device = &record.context.device;
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.draw_pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.draw_layout,
                0,
                &[self.draw_set, self.textures.set],
                &[],
            );
            device.cmd_draw_indirect(
                record.command_buffer,
                self.draw_command.buffer,
                0,
                1,
                std::mem::size_of::<[u32; 4]>() as u32,
            );
        }
    }

    fn destroy(&mut self, context: &Context) {
        destroy_image_table(context, &mut self.textures);
        unsafe {
            context.device.destroy_pipeline(self.reset_pipeline, None);
            context.device.destroy_pipeline(self.spawn_pipeline, None);
            context.device.destroy_pipeline(self.update_pipeline, None);
            context.device.destroy_pipeline(self.draw_pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.compute_layout, None);
            context
                .device
                .destroy_pipeline_layout(self.draw_layout, None);
            context.device.destroy_descriptor_pool(self.pool, None);
            context
                .device
                .destroy_descriptor_set_layout(self.compute_descriptor_layout, None);
            context
                .device
                .destroy_descriptor_set_layout(self.draw_descriptor_layout, None);
        }
        for buffer in [
            &mut self.particles,
            &mut self.emitters,
            &mut self.simulation,
            &mut self.free_indices,
            &mut self.free_count,
            &mut self.alive_indices,
            &mut self.alive_count,
            &mut self.draw_command,
            &mut self.camera,
        ] {
            destroy_buffer(context, buffer);
        }
    }
}

fn fill_free_list(pass: &ParticlesPass) {
    let indices: Vec<u32> = (0..pass.budget).collect();
    write_buffer(&pass.free_indices, &indices);
    write_buffer(&pass.free_count, &[pass.budget]);
}

fn barrier(device: &ash::Device, command_buffer: vk::CommandBuffer) {
    let memory = vk::MemoryBarrier2::default()
        .src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
        .src_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE)
        .dst_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
        .dst_access_mask(
            vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_STORAGE_WRITE,
        );
    let dependency = vk::DependencyInfo::default().memory_barriers(std::slice::from_ref(&memory));
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
}

fn draw_barrier(device: &ash::Device, command_buffer: vk::CommandBuffer) {
    let memory = vk::MemoryBarrier2::default()
        .src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
        .src_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE)
        .dst_stage_mask(crate::renderer::pass::REACHED_ON_ANY_QUEUE)
        .dst_access_mask(
            vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::INDIRECT_COMMAND_READ,
        );
    let dependency = vk::DependencyInfo::default().memory_barriers(std::slice::from_ref(&memory));
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
}

fn write_sets(context: &Context, pass: &ParticlesPass) {
    let compute_storages = [
        pass.particles.buffer,
        pass.emitters.buffer,
        pass.free_indices.buffer,
        pass.free_count.buffer,
        pass.alive_indices.buffer,
        pass.alive_count.buffer,
        pass.draw_command.buffer,
    ];
    let compute_bindings = [0u32, 1, 3, 4, 5, 6, 7];
    let simulation = vk::DescriptorBufferInfo::default()
        .buffer(pass.simulation.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let compute_infos: Vec<vk::DescriptorBufferInfo> = compute_storages
        .iter()
        .map(|buffer| {
            vk::DescriptorBufferInfo::default()
                .buffer(*buffer)
                .offset(0)
                .range(vk::WHOLE_SIZE)
        })
        .collect();

    let mut writes = vec![
        vk::WriteDescriptorSet::default()
            .dst_set(pass.compute_set)
            .dst_binding(2)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&simulation)),
    ];
    for (binding, info) in compute_bindings.iter().zip(compute_infos.iter()) {
        writes.push(
            vk::WriteDescriptorSet::default()
                .dst_set(pass.compute_set)
                .dst_binding(*binding)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(info)),
        );
    }

    let camera = vk::DescriptorBufferInfo::default()
        .buffer(pass.camera.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let draw_storages = [pass.particles.buffer, pass.alive_indices.buffer];
    let draw_infos: Vec<vk::DescriptorBufferInfo> = draw_storages
        .iter()
        .map(|buffer| {
            vk::DescriptorBufferInfo::default()
                .buffer(*buffer)
                .offset(0)
                .range(vk::WHOLE_SIZE)
        })
        .collect();
    writes.push(
        vk::WriteDescriptorSet::default()
            .dst_set(pass.draw_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&camera)),
    );
    for (index, info) in draw_infos.iter().enumerate() {
        writes.push(
            vk::WriteDescriptorSet::default()
                .dst_set(pass.draw_set)
                .dst_binding(index as u32 + 1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(info)),
        );
    }
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

fn create_storage(
    context: &Context,
    name: &str,
    size: u64,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        name,
        size,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}

fn create_shared(
    context: &Context,
    name: &str,
    size: u64,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        name,
        size,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        MemoryLocation::CpuToGpu,
    )
}

fn create_compute_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let bindings: Vec<vk::DescriptorSetLayoutBinding> = (0..8)
        .map(|binding| {
            let kind = if binding == 2 {
                vk::DescriptorType::UNIFORM_BUFFER
            } else {
                vk::DescriptorType::STORAGE_BUFFER
            };
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(kind)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE)
        })
        .collect();
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
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX),
        vk::DescriptorSetLayoutBinding::default()
            .binding(2)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX),
    ];
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

fn create_descriptor_pool(
    context: &Context,
) -> Result<vk::DescriptorPool, Box<dyn std::error::Error>> {
    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(16),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(4),
    ];
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(4)
        .pool_sizes(&sizes);
    Ok(unsafe { context.device.create_descriptor_pool(&create_info, None)? })
}

fn allocate_set(
    context: &Context,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
) -> Result<vk::DescriptorSet, Box<dyn std::error::Error>> {
    let layouts = [layout];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    Ok(unsafe { context.device.allocate_descriptor_sets(&allocate_info)?[0] })
}

fn create_pipeline_layout(
    context: &Context,
    layouts: &[vk::DescriptorSetLayout],
) -> Result<vk::PipelineLayout, Box<dyn std::error::Error>> {
    let create_info = vk::PipelineLayoutCreateInfo::default().set_layouts(layouts);
    Ok(unsafe { context.device.create_pipeline_layout(&create_info, None)? })
}

fn create_compute_pipeline(
    context: &Context,
    layout: vk::PipelineLayout,
    code: &[u8],
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let module = create_shader_module(context, code)?;
    let entry = std::ffi::CString::new("main")?;
    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(module)
        .name(&entry);
    let create_info = vk::ComputePipelineCreateInfo::default()
        .stage(stage)
        .layout(layout);
    let pipelines = unsafe {
        context
            .device
            .create_compute_pipelines(context.pipeline_cache, &[create_info], None)
    };
    unsafe { context.device.destroy_shader_module(module, None) };
    Ok(pipelines.map_err(|(_, error)| error)?[0])
}

fn create_draw_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let vertex = create_shader_module(context, include_bytes!("shaders/particle.vert.spv"))?;
    let fragment = create_shader_module(context, include_bytes!("shaders/particle.frag.spv"))?;
    let entry = std::ffi::CString::new("main")?;
    let stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex)
            .name(&entry),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment)
            .name(&entry),
    ];

    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
    let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
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
        .depth_write_enable(false)
        .depth_compare_op(vk::CompareOp::GREATER_OR_EQUAL);
    let blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE)
        .alpha_blend_op(vk::BlendOp::ADD);
    let normal_blend = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::empty());
    let attachments = [
        blend_attachment,
        normal_blend,
        vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::empty()),
    ];
    let color_blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&attachments);

    let color_formats = [setup.hdr_format, setup.normal_format, setup.motion_format];
    let mut rendering = vk::PipelineRenderingCreateInfo::default()
        .color_attachment_formats(&color_formats)
        .depth_attachment_format(setup.depth_format);

    let create_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&assembly)
        .viewport_state(&viewport_state)
        .dynamic_state(&dynamic_state)
        .rasterization_state(&rasterization)
        .multisample_state(&multisample)
        .depth_stencil_state(&depth_stencil)
        .color_blend_state(&color_blend)
        .layout(layout)
        .push_next(&mut rendering);

    let pipelines = unsafe {
        context
            .device
            .create_graphics_pipelines(context.pipeline_cache, &[create_info], None)
    };
    unsafe {
        context.device.destroy_shader_module(vertex, None);
        context.device.destroy_shader_module(fragment, None);
    }
    Ok(pipelines.map_err(|(_, error)| error)?[0])
}

pub const MAX_SPAWN_PER_FRAME: u32 = SPAWN_GROUP_SIZE;
