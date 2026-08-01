use ash::vk;
use gpu_allocator::MemoryLocation;

use crate::renderer::context::Context;
use crate::renderer::memory::{Buffer, create_buffer, destroy_buffer, write_buffer};
use crate::renderer::pass::{
    Pass, PassRecord, PassSetup, ScopeKind, ScreenTargets, create_shader_module,
};
use crate::renderer::resource::{Resource, Usage};

pub const OUTLINE_PASS: &str = "outline";

const WIDTH: f32 = 2.0;

const FIRST_WORDS: usize = 64;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct OutlineParams {
    color: [f32; 4],
    shape: [f32; 4],
}

#[derive(Default)]
pub struct OutlineFrame {
    pub color: [f32; 4],
    pub named: Vec<u32>,
}

pub struct OutlinePass {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    sets: Vec<vk::DescriptorSet>,
    buffers: Vec<Buffer>,
    names: Vec<Buffer>,
    name_words: Vec<usize>,
    sampler: vk::Sampler,
    data: OutlineFrame,
}

pub fn create_pass(setup: &PassSetup<'_>) -> Result<OutlinePass, Box<dyn std::error::Error>> {
    let context = setup.context;
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(2)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
    ];
    let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    let descriptor_layout = unsafe {
        context
            .device
            .create_descriptor_set_layout(&layout_info, None)?
    };

    let frames = setup.frames_in_flight as u32;
    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(frames),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(frames),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(frames),
    ];
    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&sizes)
        .max_sets(frames)
        .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);
    let pool = unsafe { context.device.create_descriptor_pool(&pool_info, None)? };

    let layouts = vec![descriptor_layout; setup.frames_in_flight];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let sets = unsafe { context.device.allocate_descriptor_sets(&allocate_info)? };

    let mut buffers = Vec::with_capacity(setup.frames_in_flight);
    let mut names = Vec::with_capacity(setup.frames_in_flight);
    for _ in 0..setup.frames_in_flight {
        buffers.push(create_buffer(
            context,
            "outline params",
            std::mem::size_of::<OutlineParams>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            MemoryLocation::CpuToGpu,
        )?);
        names.push(create_names(context, FIRST_WORDS)?);
    }

    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::NEAREST)
        .min_filter(vk::Filter::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE);
    let sampler = unsafe { context.device.create_sampler(&sampler_info, None)? };

    let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(std::slice::from_ref(&descriptor_layout));
    let pipeline_layout = unsafe {
        context
            .device
            .create_pipeline_layout(&pipeline_layout_info, None)?
    };
    let pipeline = create_pipeline(setup, pipeline_layout)?;

    let pass = OutlinePass {
        pipeline,
        pipeline_layout,
        descriptor_layout,
        pool,
        sets,
        buffers,
        names,
        name_words: vec![FIRST_WORDS; setup.frames_in_flight],
        sampler,
        data: OutlineFrame::default(),
    };
    write_sets(context, &pass, setup.entity_id);
    Ok(pass)
}

fn create_names(context: &Context, words: usize) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        "outline names",
        (words * std::mem::size_of::<u32>()) as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        MemoryLocation::CpuToGpu,
    )
}

fn write_names_set(context: &Context, set: vk::DescriptorSet, names: &Buffer) {
    let info = vk::DescriptorBufferInfo::default()
        .buffer(names.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let write = vk::WriteDescriptorSet::default()
        .dst_set(set)
        .dst_binding(2)
        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        .buffer_info(std::slice::from_ref(&info));
    unsafe {
        context
            .device
            .update_descriptor_sets(std::slice::from_ref(&write), &[])
    };
}

fn name_bits(named: &[u32]) -> Vec<u32> {
    let highest = named.iter().copied().max().unwrap_or(0) as usize;
    let mut bits = vec![0u32; highest / 32 + 1];
    for id in named {
        bits[*id as usize / 32] |= 1 << (*id % 32);
    }
    bits
}

pub fn hand_over(pass: &mut OutlinePass, data: OutlineFrame) {
    pass.data = data;
}

fn write_sets(context: &Context, pass: &OutlinePass, entity_id: vk::ImageView) {
    let image = vk::DescriptorImageInfo::default()
        .image_view(entity_id)
        .sampler(pass.sampler)
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    let mut writes = Vec::with_capacity(pass.sets.len() * 2);
    let buffers: Vec<vk::DescriptorBufferInfo> = pass
        .buffers
        .iter()
        .map(|buffer| {
            vk::DescriptorBufferInfo::default()
                .buffer(buffer.buffer)
                .offset(0)
                .range(vk::WHOLE_SIZE)
        })
        .collect();
    for (index, set) in pass.sets.iter().enumerate() {
        writes.push(
            vk::WriteDescriptorSet::default()
                .dst_set(*set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&image)),
        );
        writes.push(
            vk::WriteDescriptorSet::default()
                .dst_set(*set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&buffers[index])),
        );
    }
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
    for (set, names) in pass.sets.iter().zip(&pass.names) {
        write_names_set(context, *set, names);
    }
}

const READING_IDS: &[(Resource, Usage)] = &[(Resource::EntityId, Usage::Sampled)];

impl Pass for OutlinePass {
    fn name(&self) -> &'static str {
        OUTLINE_PASS
    }

    fn usage(&self, scope: ScopeKind) -> &[(Resource, Usage)] {
        match scope {
            ScopeKind::Swapchain if !self.data.named.is_empty() => READING_IDS,
            _ => &[],
        }
    }

    fn draw(&mut self, record: &mut PassRecord) {
        if self.data.named.is_empty() {
            return;
        }
        let frame_index = record.frame_index % self.buffers.len();
        let bits = name_bits(&self.data.named);
        if bits.len() > self.name_words[frame_index]
            && let Ok(grown) = create_names(record.context, bits.len())
        {
            destroy_buffer(record.context, &mut self.names[frame_index]);
            self.names[frame_index] = grown;
            self.name_words[frame_index] = bits.len();
            write_names_set(
                record.context,
                self.sets[frame_index],
                &self.names[frame_index],
            );
        }
        let words = bits.len().min(self.name_words[frame_index]);
        write_buffer(&self.names[frame_index], &bits[..words]);
        let params = OutlineParams {
            color: self.data.color,
            shape: [WIDTH, words as f32, 0.0, 0.0],
        };
        write_buffer(&self.buffers[frame_index], std::slice::from_ref(&params));

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
                &[self.sets[frame_index]],
                &[],
            );
            device.cmd_draw(record.command_buffer, 3, 1, 0, 0);
        }
    }

    fn resize(&mut self, context: &Context, targets: ScreenTargets) {
        write_sets(context, self, targets.entity_id);
    }

    fn retarget(&mut self, setup: &PassSetup<'_>) {
        let pipeline = match create_pipeline(setup, self.pipeline_layout) {
            Ok(pipeline) => pipeline,
            Err(error) => {
                tracing::error!("the outline was not made for the new screen format: {error}");
                return;
            }
        };
        unsafe { setup.context.device.destroy_pipeline(self.pipeline, None) };
        self.pipeline = pipeline;
    }

    fn destroy(&mut self, context: &Context) {
        unsafe {
            context.device.destroy_pipeline(self.pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            context.device.destroy_sampler(self.sampler, None);
            context.device.destroy_descriptor_pool(self.pool, None);
            context
                .device
                .destroy_descriptor_set_layout(self.descriptor_layout, None);
        }
        for buffer in self.buffers.iter_mut().chain(self.names.iter_mut()) {
            destroy_buffer(context, buffer);
        }
        self.buffers.clear();
        self.names.clear();
    }
}

fn create_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let vertex = create_shader_module(context, include_bytes!("shaders/fullscreen.vert.spv"))?;
    let fragment = create_shader_module(context, include_bytes!("shaders/outline.frag.spv"))?;
    let stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex)
            .name(c"main"),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment)
            .name(c"main"),
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
    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default();
    let blend_attachments = [vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)];
    let color_blend =
        vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
    let color_formats = [setup.color_format];
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

    let created = unsafe {
        context
            .device
            .create_graphics_pipelines(
                context.pipeline_cache,
                std::slice::from_ref(&create_info),
                None,
            )
            .map_err(|(_, error)| error)?
    };
    unsafe {
        context.device.destroy_shader_module(vertex, None);
        context.device.destroy_shader_module(fragment, None);
    }
    Ok(created[0])
}
