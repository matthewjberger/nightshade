use ash::vk;
use gpu_allocator::MemoryLocation;
use nightshade_render::queries::projection::matrix_columns;

use crate::renderer::context::Context;
use crate::renderer::glyph_atlas::GlyphAtlas;
use crate::renderer::memory::{Buffer, create_buffer, destroy_buffer, write_buffer};
use crate::renderer::pass::{Pass, PassRecord, PassSetup, ScopeKind, create_shader_module};
use nightshade_text::contract::data::text_data::TextVertex;

pub const TEXT_PASS: &str = "text";

const INITIAL_VERTICES: usize = 4096;
const INITIAL_INDICES: usize = 6144;
const INITIAL_CHARACTERS: usize = 1024;

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct FrameUniforms {
    view_projection: [[f32; 4]; 4],
    camera_position: [f32; 4],
    camera_right: [f32; 4],
    camera_up: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct RunPush {
    model: [[f32; 4]; 4],
    color: [f32; 4],
    outline_color: [f32; 4],
    settings: [f32; 4],
}

pub struct TextRun {
    pub model: [[f32; 4]; 4],
    pub color: [f32; 4],
    pub outline_color: [f32; 4],
    pub outline_width: f32,
    pub smoothing: f32,
    pub billboard: bool,
    pub vertex_offset: i32,
    pub index_offset: u32,
    pub index_count: u32,
    pub character_offset: u32,
}

#[derive(Default)]
pub struct TextFrame {
    pub vertices: Vec<TextVertex>,
    pub indices: Vec<u32>,
    pub character_colors: Vec<[f32; 4]>,
    pub runs: Vec<TextRun>,
}

struct FrameResources {
    globals: Buffer,
    vertices: Buffer,
    indices: Buffer,
    character_colors: Buffer,
    set: vk::DescriptorSet,
    vertex_capacity: usize,
    index_capacity: usize,
    character_capacity: usize,
}

pub struct TextPass {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    frames: Vec<FrameResources>,
    bound_revision: u32,
    data: TextFrame,
}

pub fn create_pass(setup: &PassSetup<'_>) -> Result<TextPass, Box<dyn std::error::Error>> {
    let context = setup.context;
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
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

    let count = setup.frames_in_flight as u32;
    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(count),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(count),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(count),
    ];
    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&sizes)
        .max_sets(count);
    let pool = unsafe { context.device.create_descriptor_pool(&pool_info, None)? };

    let range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
        .offset(0)
        .size(std::mem::size_of::<RunPush>() as u32);
    let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(std::slice::from_ref(&descriptor_layout))
        .push_constant_ranges(std::slice::from_ref(&range));
    let pipeline_layout = unsafe {
        context
            .device
            .create_pipeline_layout(&pipeline_layout_info, None)?
    };
    let pipeline = create_pipeline(setup, pipeline_layout)?;

    let mut frames = Vec::with_capacity(setup.frames_in_flight);
    for _ in 0..setup.frames_in_flight {
        frames.push(create_frame(
            context,
            pool,
            descriptor_layout,
            setup.glyphs,
        )?);
    }

    Ok(TextPass {
        pipeline,
        pipeline_layout,
        descriptor_layout,
        pool,
        frames,
        bound_revision: crate::renderer::glyph_atlas::revision(setup.glyphs),
        data: TextFrame::default(),
    })
}

pub fn hand_over(pass: &mut TextPass, data: TextFrame) {
    pass.data = data;
}

fn create_frame(
    context: &Context,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    glyphs: &GlyphAtlas,
) -> Result<FrameResources, Box<dyn std::error::Error>> {
    let layouts = [layout];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let set = unsafe { context.device.allocate_descriptor_sets(&allocate_info)?[0] };

    let frame = FrameResources {
        globals: create_buffer(
            context,
            "text globals",
            std::mem::size_of::<FrameUniforms>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            MemoryLocation::CpuToGpu,
        )?,
        vertices: create_buffer(
            context,
            "text vertices",
            (INITIAL_VERTICES * std::mem::size_of::<TextVertex>()) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            MemoryLocation::CpuToGpu,
        )?,
        indices: create_buffer(
            context,
            "text indices",
            (INITIAL_INDICES * 4) as u64,
            vk::BufferUsageFlags::INDEX_BUFFER,
            MemoryLocation::CpuToGpu,
        )?,
        character_colors: create_buffer(
            context,
            "text character colors",
            (INITIAL_CHARACTERS * 16) as u64,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            MemoryLocation::CpuToGpu,
        )?,
        set,
        vertex_capacity: INITIAL_VERTICES,
        index_capacity: INITIAL_INDICES,
        character_capacity: INITIAL_CHARACTERS,
    };
    bind_frame(context, &frame, glyphs);
    Ok(frame)
}

fn bind_frame(context: &Context, frame: &FrameResources, glyphs: &GlyphAtlas) {
    let globals = vk::DescriptorBufferInfo::default()
        .buffer(frame.globals.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let colors = vk::DescriptorBufferInfo::default()
        .buffer(frame.character_colors.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let atlas = vk::DescriptorImageInfo::default()
        .image_view(glyphs.view)
        .sampler(glyphs.sampler)
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    let writes = [
        vk::WriteDescriptorSet::default()
            .dst_set(frame.set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&globals)),
        vk::WriteDescriptorSet::default()
            .dst_set(frame.set)
            .dst_binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&atlas)),
        vk::WriteDescriptorSet::default()
            .dst_set(frame.set)
            .dst_binding(2)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&colors)),
    ];
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

impl Pass for TextPass {
    fn name(&self) -> &'static str {
        TEXT_PASS
    }

    fn draw(&mut self, record: &mut PassRecord) {
        if record.scope != ScopeKind::Color || self.data.runs.is_empty() {
            return;
        }
        let frame_index = record.frame_index % self.frames.len();
        if let Err(error) = self.upload(record.context, record.glyphs, frame_index) {
            tracing::error!("the text was not uploaded: {error}");
            return;
        }
        let Some(view) = record.view else {
            return;
        };

        let frame = &self.frames[frame_index];
        write_buffer(
            &frame.globals,
            std::slice::from_ref(&FrameUniforms {
                view_projection: matrix_columns(&view.view_projection),
                camera_position: [
                    view.camera_position.x,
                    view.camera_position.y,
                    view.camera_position.z,
                    1.0,
                ],
                camera_right: [
                    view.camera_right.x,
                    view.camera_right.y,
                    view.camera_right.z,
                    0.0,
                ],
                camera_up: [view.camera_up.x, view.camera_up.y, view.camera_up.z, 0.0],
            }),
        );
        write_buffer(&frame.vertices, &self.data.vertices);
        write_buffer(&frame.indices, &self.data.indices);
        write_buffer(&frame.character_colors, &self.data.character_colors);

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
            device.cmd_bind_vertex_buffers(
                record.command_buffer,
                0,
                &[frame.vertices.buffer],
                &[0],
            );
            device.cmd_bind_index_buffer(
                record.command_buffer,
                frame.indices.buffer,
                0,
                vk::IndexType::UINT32,
            );
        }

        for run in &self.data.runs {
            let push = RunPush {
                model: run.model,
                color: run.color,
                outline_color: run.outline_color,
                settings: [
                    run.outline_width,
                    run.smoothing,
                    if run.billboard { 1.0 } else { 0.0 },
                    run.character_offset as f32,
                ],
            };
            unsafe {
                device.cmd_push_constants(
                    record.command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    bytemuck::bytes_of(&push),
                );
                device.cmd_draw_indexed(
                    record.command_buffer,
                    run.index_count,
                    1,
                    run.index_offset,
                    run.vertex_offset,
                    0,
                );
            }
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
            destroy_buffer(context, &mut frame.globals);
            destroy_buffer(context, &mut frame.vertices);
            destroy_buffer(context, &mut frame.indices);
            destroy_buffer(context, &mut frame.character_colors);
        }
        self.frames.clear();
    }
}

impl TextPass {
    fn upload(
        &mut self,
        context: &Context,
        glyphs: &GlyphAtlas,
        frame_index: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let revision = crate::renderer::glyph_atlas::revision(glyphs);
        if revision != self.bound_revision {
            for frame in &self.frames {
                bind_frame(context, frame, glyphs);
            }
            self.bound_revision = revision;
        }

        let mut rebind = false;
        let frame = &mut self.frames[frame_index];
        if frame.vertex_capacity < self.data.vertices.len() {
            let capacity = self.data.vertices.len().next_power_of_two();
            let buffer = create_buffer(
                context,
                "text vertices",
                (capacity * std::mem::size_of::<TextVertex>()) as u64,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                MemoryLocation::CpuToGpu,
            )?;
            destroy_buffer(context, &mut frame.vertices);
            frame.vertices = buffer;
            frame.vertex_capacity = capacity;
        }
        if frame.index_capacity < self.data.indices.len() {
            let capacity = self.data.indices.len().next_power_of_two();
            let buffer = create_buffer(
                context,
                "text indices",
                (capacity * 4) as u64,
                vk::BufferUsageFlags::INDEX_BUFFER,
                MemoryLocation::CpuToGpu,
            )?;
            destroy_buffer(context, &mut frame.indices);
            frame.indices = buffer;
            frame.index_capacity = capacity;
        }
        if frame.character_capacity < self.data.character_colors.len() {
            let capacity = self.data.character_colors.len().next_power_of_two();
            let buffer = create_buffer(
                context,
                "text character colors",
                (capacity * 16) as u64,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                MemoryLocation::CpuToGpu,
            )?;
            destroy_buffer(context, &mut frame.character_colors);
            frame.character_colors = buffer;
            frame.character_capacity = capacity;
            rebind = true;
        }
        if rebind {
            bind_frame(context, &self.frames[frame_index], glyphs);
        }
        Ok(())
    }
}

fn create_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let vertex = create_shader_module(context, include_bytes!("shaders/text.vert.spv"))?;
    let fragment = create_shader_module(context, include_bytes!("shaders/text.frag.spv"))?;
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
    let binding = vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(std::mem::size_of::<TextVertex>() as u32)
        .input_rate(vk::VertexInputRate::VERTEX);
    let attributes = [
        vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0),
        vk::VertexInputAttributeDescription::default()
            .location(1)
            .binding(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(12),
        vk::VertexInputAttributeDescription::default()
            .location(2)
            .binding(0)
            .format(vk::Format::R32_UINT)
            .offset(20),
        vk::VertexInputAttributeDescription::default()
            .location(3)
            .binding(0)
            .format(vk::Format::R32_UINT)
            .offset(24),
    ];
    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(std::slice::from_ref(&binding))
        .vertex_attribute_descriptions(&attributes);
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
    let blend = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .alpha_blend_op(vk::BlendOp::ADD);
    let blend_attachments = [
        blend,
        vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::empty()),
        vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::empty()),
    ];
    let color_blend =
        vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachments);
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
