use ash::vk;

use crate::renderer::context::Context;
use crate::renderer::memory::{Buffer, create_buffer, destroy_buffer, write_buffer};
use crate::renderer::pass::{Pass, PassRecord, PassSetup, create_shader_module};
use gpu_allocator::MemoryLocation;

pub const SKY_PASS: &str = "sky";

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSky {
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub settings: [f32; 4],
    pub air: [f32; 4],
    pub fog_color: [f32; 4],
}

pub const ATMOSPHERE_NONE: u32 = 0;
pub const ATMOSPHERE_SKY: u32 = 1;
pub const ATMOSPHERE_CLOUDY: u32 = 2;
pub const ATMOSPHERE_SPACE: u32 = 3;
pub const ATMOSPHERE_NEBULA: u32 = 4;
pub const ATMOSPHERE_SUNSET: u32 = 5;
pub const ATMOSPHERE_DAY_NIGHT: u32 = 6;
pub const ATMOSPHERE_HDR: u32 = 7;

struct Frame {
    uniform: Buffer,
    set: vk::DescriptorSet,
}

pub struct SkyPass {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    frames: Vec<Frame>,
    data: GpuSky,
}

pub fn create_pass(
    setup: &PassSetup<'_>,
    environment: vk::ImageView,
    environment_sampler: vk::Sampler,
    air: vk::DescriptorImageInfo,
) -> Result<SkyPass, Box<dyn std::error::Error>> {
    let context = setup.context;
    let descriptor_layout = create_descriptor_layout(context)?;
    let pool = create_descriptor_pool(context, setup.frames_in_flight)?;

    let mut frames = Vec::with_capacity(setup.frames_in_flight);
    for _ in 0..setup.frames_in_flight {
        frames.push(create_frame(
            context,
            pool,
            descriptor_layout,
            environment,
            environment_sampler,
            air,
        )?);
    }

    let layouts = [descriptor_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
    let pipeline_layout = unsafe { context.device.create_pipeline_layout(&layout_info, None)? };
    let pipeline = create_pipeline(setup, pipeline_layout)?;

    Ok(SkyPass {
        pipeline,
        pipeline_layout,
        descriptor_layout,
        pool,
        frames,
        data: GpuSky::default(),
    })
}

pub fn hand_over(pass: &mut SkyPass, data: GpuSky) {
    pass.data = data;
}

impl Pass for SkyPass {
    fn name(&self) -> &'static str {
        SKY_PASS
    }

    fn draw(&mut self, record: &mut PassRecord) {
        if self.data.settings[2] as u32 == ATMOSPHERE_NONE && self.data.air[2] == 0.0 {
            return;
        }
        let Some(frame) = self.frames.get(record.frame_index) else {
            return;
        };
        write_buffer(&frame.uniform, std::slice::from_ref(&self.data));

        let device = &record.context.device;
        unsafe {
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[frame.set],
                &[],
            );
            device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
        }
    }

    fn destroy(&mut self, context: &Context) {
        unsafe {
            context.device.destroy_pipeline(self.pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            context.device.destroy_descriptor_pool(self.pool, None);
            context
                .device
                .destroy_descriptor_set_layout(self.descriptor_layout, None);
        }
        for frame in &mut self.frames {
            destroy_buffer(context, &mut frame.uniform);
        }
    }
}

fn create_frame(
    context: &Context,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    environment: vk::ImageView,
    environment_sampler: vk::Sampler,
    air: vk::DescriptorImageInfo,
) -> Result<Frame, Box<dyn std::error::Error>> {
    let uniform = create_buffer(
        context,
        "sky",
        std::mem::size_of::<GpuSky>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;
    let layouts = [layout];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let set = unsafe { context.device.allocate_descriptor_sets(&allocate_info)?[0] };

    let info = vk::DescriptorBufferInfo::default()
        .buffer(uniform.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let cube = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(environment)
        .sampler(environment_sampler);
    let writes = [
        vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&info)),
        vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&cube)),
        vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(2)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&air)),
    ];
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };

    Ok(Frame { uniform, set })
}

fn create_descriptor_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(2)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
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
    frames: usize,
) -> Result<vk::DescriptorPool, Box<dyn std::error::Error>> {
    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(frames as u32),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(frames as u32 * 2),
    ];
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(frames as u32)
        .pool_sizes(&sizes);
    Ok(unsafe { context.device.create_descriptor_pool(&create_info, None)? })
}

fn create_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let vertex = create_shader_module(context, include_bytes!("shaders/sky.vert.spv"))?;
    let fragment = create_shader_module(context, include_bytes!("shaders/sky.frag.spv"))?;
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
        .depth_test_enable(false)
        .depth_write_enable(false);
    let blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA);
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
    let pipeline = pipelines.map_err(|(_, error)| error)?[0];
    Ok(pipeline)
}
