use ash::vk;
use gpu_allocator::MemoryLocation;

use crate::renderer::context::Context;
use crate::renderer::memory::{Buffer, create_buffer, destroy_buffer, write_buffer};
use crate::renderer::pass::{
    Pass, PassRecord, PassSetup, ScopeKind, ScreenTargets, create_shader_module,
};
use crate::renderer::resource::{Resource, Usage};

pub const DECALS_PASS: &str = "decals";

const MAX_DECALS: u64 = 1 << 10;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuDecal {
    pub model: [[f32; 4]; 4],
    pub inverse_model: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub emissive: [f32; 4],
    pub size_depth: [f32; 4],
    pub params: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Camera {
    view_projection: [[f32; 4]; 4],
    inverse_view_projection: [[f32; 4]; 4],
    camera_position: [f32; 4],
    screen: [f32; 4],
}

#[derive(Default)]
pub struct DecalFrameData {
    pub decals: Vec<GpuDecal>,
    pub view_projection: [[f32; 4]; 4],
    pub inverse_view_projection: [[f32; 4]; 4],
    pub camera_position: [f32; 4],
    pub screen: [f32; 4],
}

pub struct DecalsPass {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    set: vk::DescriptorSet,
    camera: Buffer,
    decals: Buffer,
    sampler: vk::Sampler,
    textures: vk::DescriptorSet,
    data: DecalFrameData,
}

pub fn create_pass(
    setup: &PassSetup<'_>,
    texture_layout: vk::DescriptorSetLayout,
) -> Result<DecalsPass, Box<dyn std::error::Error>> {
    let context = setup.context;
    let depth_view = setup.depth;
    let descriptor_layout = create_descriptor_layout(context)?;
    let pool = create_descriptor_pool(context)?;
    let set = allocate_set(context, pool, descriptor_layout)?;
    let sampler = create_sampler(context)?;

    let camera = create_buffer(
        context,
        "decal camera",
        std::mem::size_of::<Camera>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;
    let decals = create_buffer(
        context,
        "decals",
        MAX_DECALS * std::mem::size_of::<GpuDecal>() as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;

    let layouts = [descriptor_layout, texture_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
    let pipeline_layout = unsafe { context.device.create_pipeline_layout(&layout_info, None)? };
    let pipeline = create_pipeline(setup, pipeline_layout)?;

    let pass = DecalsPass {
        pipeline,
        pipeline_layout,
        descriptor_layout,
        pool,
        set,
        camera,
        decals,
        sampler,
        textures: vk::DescriptorSet::null(),
        data: DecalFrameData::default(),
    };
    write_set(context, &pass, depth_view);
    Ok(pass)
}

pub fn hand_over(pass: &mut DecalsPass, data: DecalFrameData, textures: vk::DescriptorSet) {
    pass.data = data;
    pass.textures = textures;
}

const LANDING: &[(Resource, Usage)] = &[(Resource::Depth, Usage::Sampled)];

impl Pass for DecalsPass {
    fn name(&self) -> &'static str {
        DECALS_PASS
    }

    fn usage(&self, scope: ScopeKind) -> &[(Resource, Usage)] {
        match scope {
            ScopeKind::Decal => LANDING,
            _ => &[],
        }
    }

    fn draw(&mut self, record: &mut PassRecord) {
        if self.data.decals.is_empty() || self.textures == vk::DescriptorSet::null() {
            return;
        }
        write_buffer(
            &self.camera,
            std::slice::from_ref(&Camera {
                view_projection: self.data.view_projection,
                inverse_view_projection: self.data.inverse_view_projection,
                camera_position: self.data.camera_position,
                screen: self.data.screen,
            }),
        );
        let count = self.data.decals.len().min(MAX_DECALS as usize);
        write_buffer(&self.decals, &self.data.decals[..count]);

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
                &[self.set, self.textures],
                &[],
            );
            device.cmd_draw(record.command_buffer, 36, count as u32, 0, 0);
        }
    }

    fn resize(&mut self, context: &Context, targets: ScreenTargets) {
        write_set(context, self, targets.depth);
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
            context.device.destroy_sampler(self.sampler, None);
        }
        destroy_buffer(context, &mut self.camera);
        destroy_buffer(context, &mut self.decals);
    }
}

pub fn write_set(context: &Context, pass: &DecalsPass, depth_view: vk::ImageView) {
    let camera = vk::DescriptorBufferInfo::default()
        .buffer(pass.camera.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let decals = vk::DescriptorBufferInfo::default()
        .buffer(pass.decals.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let depth = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(depth_view)
        .sampler(pass.sampler);
    let writes = [
        vk::WriteDescriptorSet::default()
            .dst_set(pass.set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&camera)),
        vk::WriteDescriptorSet::default()
            .dst_set(pass.set)
            .dst_binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&decals)),
        vk::WriteDescriptorSet::default()
            .dst_set(pass.set)
            .dst_binding(2)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&depth)),
    ];
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
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
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
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
) -> Result<vk::DescriptorPool, Box<dyn std::error::Error>> {
    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(2),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(2),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(2),
    ];
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(2)
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

fn create_sampler(context: &Context) -> Result<vk::Sampler, Box<dyn std::error::Error>> {
    let create_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::NEAREST)
        .min_filter(vk::Filter::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE);
    Ok(unsafe { context.device.create_sampler(&create_info, None)? })
}

fn create_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let vertex = create_shader_module(context, include_bytes!("shaders/decal.vert.spv"))?;
    let fragment = create_shader_module(context, include_bytes!("shaders/decal.frag.spv"))?;
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
        .cull_mode(vk::CullModeFlags::FRONT)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);
    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(false)
        .depth_write_enable(false);
    let blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .alpha_blend_op(vk::BlendOp::ADD);
    let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(std::slice::from_ref(&blend_attachment));

    let color_formats = [setup.hdr_format];
    let mut rendering =
        vk::PipelineRenderingCreateInfo::default().color_attachment_formats(&color_formats);

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
