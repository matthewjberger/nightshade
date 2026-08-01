use ash::vk;
use gpu_allocator::MemoryLocation;

use crate::renderer::context::Context;
use crate::renderer::memory::{Buffer, create_buffer, destroy_buffer, write_buffer};
use crate::renderer::pass::{Pass, PassRecord, PassSetup, create_shader_module};

pub const LINES_PASS: &str = "lines";

const INITIAL_END_CAPACITY: u64 = 1 << 12;
const INITIAL_RECORD_CAPACITY: u64 = 1 << 8;
const ENDS_PER_VOLUME: u64 = 24;
const ENDS_PER_NORMAL: u64 = 2;
const GROUP_SIZE: u32 = 64;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuVolume {
    pub transform: [[f32; 4]; 4],
    pub center: [f32; 4],
    pub half_extents: [f32; 4],
    pub orientation: [f32; 4],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuNormal {
    pub transform: [[f32; 4]; 4],
    pub position: [f32; 4],
    pub direction: [f32; 4],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct LineParams {
    view_projection: [[f32; 4]; 4],
    on_top: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Batch {
    count: u32,
    first_end: u32,
}

#[derive(Default)]
pub struct LineFrame {
    pub tested: Vec<LineVertex>,
    pub on_top: Vec<LineVertex>,
    pub volumes: Vec<GpuVolume>,
    pub normals: Vec<GpuNormal>,
    pub view_projection: [[f32; 4]; 4],
}

struct FrameBuffer {
    ends: Buffer,
    end_capacity: u64,
    plain: Buffer,
    plain_capacity: u64,
    volumes: Buffer,
    volume_capacity: u64,
    normals: Buffer,
    normal_capacity: u64,
    set: vk::DescriptorSet,
    bound: bool,
}

pub struct LinesPass {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    volume_pipeline: vk::Pipeline,
    normal_pipeline: vk::Pipeline,
    expand_layout: vk::PipelineLayout,
    descriptor_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    frames: Vec<FrameBuffer>,
    data: LineFrame,
}

pub fn create_pass(setup: &PassSetup<'_>) -> Result<LinesPass, Box<dyn std::error::Error>> {
    let context = setup.context;
    let range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .offset(0)
        .size(std::mem::size_of::<LineParams>() as u32);
    let layout_info =
        vk::PipelineLayoutCreateInfo::default().push_constant_ranges(std::slice::from_ref(&range));
    let pipeline_layout = unsafe { context.device.create_pipeline_layout(&layout_info, None)? };
    let pipeline = create_pipeline(setup, pipeline_layout)?;

    let descriptor_layout = create_descriptor_layout(context)?;
    let expand_range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::COMPUTE)
        .offset(0)
        .size(std::mem::size_of::<Batch>() as u32);
    let layouts = [descriptor_layout];
    let expand_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(&layouts)
        .push_constant_ranges(std::slice::from_ref(&expand_range));
    let expand_layout = unsafe { context.device.create_pipeline_layout(&expand_info, None)? };
    let volume_pipeline = create_expand_pipeline(
        context,
        expand_layout,
        include_bytes!("shaders/line_volume.comp.spv"),
    )?;
    let normal_pipeline = create_expand_pipeline(
        context,
        expand_layout,
        include_bytes!("shaders/line_normal.comp.spv"),
    )?;

    let size = vk::DescriptorPoolSize::default()
        .ty(vk::DescriptorType::STORAGE_BUFFER)
        .descriptor_count((3 * setup.frames_in_flight) as u32);
    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(std::slice::from_ref(&size))
        .max_sets(setup.frames_in_flight as u32);
    let descriptor_pool = unsafe { context.device.create_descriptor_pool(&pool_info, None)? };

    let mut frames = Vec::with_capacity(setup.frames_in_flight);
    for _ in 0..setup.frames_in_flight {
        frames.push(create_frame(context, descriptor_pool, descriptor_layout)?);
    }

    Ok(LinesPass {
        pipeline,
        pipeline_layout,
        volume_pipeline,
        normal_pipeline,
        expand_layout,
        descriptor_layout,
        descriptor_pool,
        frames,
        data: LineFrame::default(),
    })
}

pub fn hand_over(pass: &mut LinesPass, data: LineFrame) {
    pass.data = data;
}

impl LinesPass {
    fn tested_ends(&self) -> u64 {
        self.data.tested.len() as u64
            + self.data.volumes.len() as u64 * ENDS_PER_VOLUME
            + self.data.normals.len() as u64 * ENDS_PER_NORMAL
    }

    fn total_ends(&self) -> u64 {
        self.tested_ends() + self.data.on_top.len() as u64
    }
}

impl Pass for LinesPass {
    fn name(&self) -> &'static str {
        LINES_PASS
    }

    fn compute(&mut self, record: &mut PassRecord) {
        let total = self.total_ends();
        let tested = self.tested_ends();
        let tested_plain = self.data.tested.len() as u64;
        let on_top_plain = self.data.on_top.len() as u64;
        let plain = tested_plain + on_top_plain;
        if total == 0 {
            return;
        }
        let Some(frame) = self.frames.get_mut(record.frame_index) else {
            return;
        };
        if let Err(error) = reserve(
            record.context,
            frame,
            total,
            plain,
            self.data.volumes.len() as u64,
            self.data.normals.len() as u64,
        ) {
            tracing::error!("the line buffers were not grown: {error}");
            return;
        }
        if !frame.bound {
            write_set(record.context, frame);
            frame.bound = true;
        }

        let mut ends = Vec::with_capacity(plain as usize);
        ends.extend_from_slice(&self.data.tested);
        ends.extend_from_slice(&self.data.on_top);
        write_buffer(&frame.plain, &ends);
        write_buffer(&frame.volumes, &self.data.volumes);
        write_buffer(&frame.normals, &self.data.normals);

        let stride = std::mem::size_of::<LineVertex>() as u64;
        let device = &record.context.device;
        let mut regions: Vec<vk::BufferCopy> = Vec::new();
        if tested_plain > 0 {
            regions.push(
                vk::BufferCopy::default()
                    .src_offset(0)
                    .dst_offset(0)
                    .size(tested_plain * stride),
            );
        }
        if on_top_plain > 0 {
            regions.push(
                vk::BufferCopy::default()
                    .src_offset(tested_plain * stride)
                    .dst_offset(tested * stride)
                    .size(on_top_plain * stride),
            );
        }
        unsafe {
            if !regions.is_empty() {
                device.cmd_copy_buffer(
                    record.command_buffer,
                    frame.plain.buffer,
                    frame.ends.buffer,
                    &regions,
                );
                barrier(
                    device,
                    record.command_buffer,
                    frame.ends.buffer,
                    vk::PipelineStageFlags2::TRANSFER,
                    vk::AccessFlags2::TRANSFER_WRITE,
                    vk::PipelineStageFlags2::COMPUTE_SHADER,
                    vk::AccessFlags2::SHADER_STORAGE_WRITE,
                );
            }
            device.cmd_bind_descriptor_sets(
                record.command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.expand_layout,
                0,
                &[frame.set],
                &[],
            );
        }

        let rounds = [
            (
                self.volume_pipeline,
                self.data.volumes.len() as u32,
                tested_plain as u32,
            ),
            (
                self.normal_pipeline,
                self.data.normals.len() as u32,
                (tested_plain + self.data.volumes.len() as u64 * ENDS_PER_VOLUME) as u32,
            ),
        ];
        for (pipeline, count, first_end) in rounds {
            if count == 0 {
                continue;
            }
            let batch = Batch { count, first_end };
            unsafe {
                device.cmd_bind_pipeline(
                    record.command_buffer,
                    vk::PipelineBindPoint::COMPUTE,
                    pipeline,
                );
                device.cmd_push_constants(
                    record.command_buffer,
                    self.expand_layout,
                    vk::ShaderStageFlags::COMPUTE,
                    0,
                    bytemuck::bytes_of(&batch),
                );
                device.cmd_dispatch(record.command_buffer, count.div_ceil(GROUP_SIZE), 1, 1);
            }
        }

        barrier(
            device,
            record.command_buffer,
            frame.ends.buffer,
            vk::PipelineStageFlags2::COMPUTE_SHADER | vk::PipelineStageFlags2::TRANSFER,
            vk::AccessFlags2::SHADER_STORAGE_WRITE | vk::AccessFlags2::TRANSFER_WRITE,
            crate::renderer::pass::REACHED_ON_ANY_QUEUE,
            vk::AccessFlags2::VERTEX_ATTRIBUTE_READ,
        );
    }

    fn draw(&mut self, record: &mut PassRecord) {
        let tested = self.tested_ends();
        let total = self.total_ends();
        if total == 0 {
            return;
        }
        let Some(frame) = self.frames.get(record.frame_index) else {
            return;
        };
        if total > frame.end_capacity {
            return;
        }

        let device = &record.context.device;
        unsafe {
            device.cmd_bind_vertex_buffers(record.command_buffer, 0, &[frame.ends.buffer], &[0]);
            device.cmd_bind_pipeline(
                record.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
        }
        let runs = [
            (0u32, tested as u32, 0u32),
            (tested as u32, self.data.on_top.len() as u32, 1u32),
        ];
        for (first, count, on_top) in runs {
            if count == 0 {
                continue;
            }
            let params = LineParams {
                view_projection: self.data.view_projection,
                on_top,
            };
            unsafe {
                device.cmd_push_constants(
                    record.command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    bytemuck::bytes_of(&params),
                );
                device.cmd_draw(record.command_buffer, count, 1, first, 0);
            }
        }
    }

    fn destroy(&mut self, context: &Context) {
        unsafe {
            context.device.destroy_pipeline(self.pipeline, None);
            context.device.destroy_pipeline(self.volume_pipeline, None);
            context.device.destroy_pipeline(self.normal_pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            context
                .device
                .destroy_pipeline_layout(self.expand_layout, None);
            context
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            context
                .device
                .destroy_descriptor_set_layout(self.descriptor_layout, None);
        }
        for frame in &mut self.frames {
            destroy_buffer(context, &mut frame.ends);
            destroy_buffer(context, &mut frame.plain);
            destroy_buffer(context, &mut frame.volumes);
            destroy_buffer(context, &mut frame.normals);
        }
    }
}

fn barrier(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    buffer: vk::Buffer,
    source_stage: vk::PipelineStageFlags2,
    source_access: vk::AccessFlags2,
    destination_stage: vk::PipelineStageFlags2,
    destination_access: vk::AccessFlags2,
) {
    let memory = vk::BufferMemoryBarrier2::default()
        .src_stage_mask(source_stage)
        .src_access_mask(source_access)
        .dst_stage_mask(destination_stage)
        .dst_access_mask(destination_access)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .buffer(buffer)
        .size(vk::WHOLE_SIZE);
    let dependency =
        vk::DependencyInfo::default().buffer_memory_barriers(std::slice::from_ref(&memory));
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
}

fn reserve(
    context: &Context,
    frame: &mut FrameBuffer,
    ends: u64,
    plain: u64,
    volumes: u64,
    normals: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    if ends > frame.end_capacity {
        let capacity = ends.next_power_of_two();
        let grown = create_end_buffer(context, capacity)?;
        destroy_buffer(context, &mut frame.ends);
        frame.ends = grown;
        frame.end_capacity = capacity;
        frame.bound = false;
    }
    if plain > frame.plain_capacity {
        let capacity = plain.next_power_of_two();
        let grown = create_plain_buffer(context, capacity)?;
        destroy_buffer(context, &mut frame.plain);
        frame.plain = grown;
        frame.plain_capacity = capacity;
    }
    if volumes > frame.volume_capacity {
        let capacity = volumes.next_power_of_two();
        let grown = create_record_buffer(
            context,
            "scene line volumes",
            capacity * std::mem::size_of::<GpuVolume>() as u64,
        )?;
        destroy_buffer(context, &mut frame.volumes);
        frame.volumes = grown;
        frame.volume_capacity = capacity;
        frame.bound = false;
    }
    if normals > frame.normal_capacity {
        let capacity = normals.next_power_of_two();
        let grown = create_record_buffer(
            context,
            "scene line normals",
            capacity * std::mem::size_of::<GpuNormal>() as u64,
        )?;
        destroy_buffer(context, &mut frame.normals);
        frame.normals = grown;
        frame.normal_capacity = capacity;
        frame.bound = false;
    }
    Ok(())
}

fn create_frame(
    context: &Context,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
) -> Result<FrameBuffer, Box<dyn std::error::Error>> {
    let layouts = [layout];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let set = unsafe { context.device.allocate_descriptor_sets(&allocate_info)?[0] };
    Ok(FrameBuffer {
        ends: create_end_buffer(context, INITIAL_END_CAPACITY)?,
        end_capacity: INITIAL_END_CAPACITY,
        plain: create_plain_buffer(context, INITIAL_END_CAPACITY)?,
        plain_capacity: INITIAL_END_CAPACITY,
        volumes: create_record_buffer(
            context,
            "scene line volumes",
            INITIAL_RECORD_CAPACITY * std::mem::size_of::<GpuVolume>() as u64,
        )?,
        volume_capacity: INITIAL_RECORD_CAPACITY,
        normals: create_record_buffer(
            context,
            "scene line normals",
            INITIAL_RECORD_CAPACITY * std::mem::size_of::<GpuNormal>() as u64,
        )?,
        normal_capacity: INITIAL_RECORD_CAPACITY,
        set,
        bound: false,
    })
}

fn write_set(context: &Context, frame: &FrameBuffer) {
    let bound = [
        (0u32, frame.ends.buffer, frame.ends.size),
        (1, frame.volumes.buffer, frame.volumes.size),
        (2, frame.normals.buffer, frame.normals.size),
    ];
    let infos: Vec<vk::DescriptorBufferInfo> = bound
        .iter()
        .map(|(_, buffer, size)| {
            vk::DescriptorBufferInfo::default()
                .buffer(*buffer)
                .offset(0)
                .range(*size)
        })
        .collect();
    let writes: Vec<vk::WriteDescriptorSet> = bound
        .iter()
        .zip(infos.iter())
        .map(|((binding, _, _), info)| {
            vk::WriteDescriptorSet::default()
                .dst_set(frame.set)
                .dst_binding(*binding)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(info))
        })
        .collect();
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

fn create_descriptor_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let bindings: Vec<vk::DescriptorSetLayoutBinding> = (0..3)
        .map(|binding| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
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

fn create_end_buffer(
    context: &Context,
    capacity: u64,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        "scene lines",
        capacity * std::mem::size_of::<LineVertex>() as u64,
        vk::BufferUsageFlags::VERTEX_BUFFER
            | vk::BufferUsageFlags::STORAGE_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}

fn create_plain_buffer(
    context: &Context,
    capacity: u64,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        "scene line ends",
        capacity * std::mem::size_of::<LineVertex>() as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )
}

fn create_record_buffer(
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

fn create_expand_pipeline(
    context: &Context,
    layout: vk::PipelineLayout,
    code: &[u8],
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let module = create_shader_module(context, code)?;
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
                context.pipeline_cache,
                std::slice::from_ref(&create_info),
                None,
            )
            .map_err(|(_, error)| error)?
    };
    unsafe { context.device.destroy_shader_module(module, None) };
    Ok(created[0])
}

fn create_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let vertex = create_shader_module(context, include_bytes!("shaders/line.vert.spv"))?;
    let fragment = create_shader_module(context, include_bytes!("shaders/line.frag.spv"))?;
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

    let binding = vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(std::mem::size_of::<LineVertex>() as u32)
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
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(std::mem::size_of::<[f32; 3]>() as u32),
    ];
    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(std::slice::from_ref(&binding))
        .vertex_attribute_descriptions(&attributes);
    let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::LINE_LIST);
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
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
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
