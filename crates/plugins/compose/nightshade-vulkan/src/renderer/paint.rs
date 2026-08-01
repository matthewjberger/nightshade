use crate::renderer::context::Context;
use crate::renderer::glyph_atlas::GlyphAtlas;
use crate::renderer::image_table::{
    ImageTable, MAX_PAINT_IMAGES, create_image_table, destroy_image_table,
};
use crate::renderer::memory::{Buffer, create_buffer, destroy_buffer, write_buffer};
use crate::renderer::pass::{Pass, PassRecord, PassSetup, create_shader_module};
use ash::vk;
use gpu_allocator::MemoryLocation;
use nightshade_paint::{PaintImageInstance, PaintRectInstance, TextInstanceData, UiTextVertex};

pub const PAINT_PASS: &str = "paint_pass";

const INITIAL_RECT_CAPACITY: usize = 4096;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniforms {
    projection: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct QuadVertex {
    position: [f32; 2],
}

struct FrameResources {
    globals: Buffer,
    instances: Buffer,
    set: vk::DescriptorSet,
    capacity: usize,
}

struct TextFrame {
    globals: Buffer,
    instances: Buffer,
    character_colors: Buffer,
    character_backgrounds: Buffer,
    vertices: Buffer,
    indices: Buffer,
    set: vk::DescriptorSet,
    instance_capacity: usize,
    vertex_capacity: usize,
    index_capacity: usize,
    character_capacity: usize,
}

#[derive(Default)]
pub struct TextDraw {
    pub vertices: Vec<UiTextVertex>,
    pub indices: Vec<u32>,
    pub instances: Vec<TextInstanceData>,
    pub character_colors: Vec<[f32; 4]>,
    pub character_backgrounds: Vec<[f32; 4]>,
    pub label_ranges: Vec<LabelRange>,
}

#[derive(Clone, Copy)]
pub struct LabelRange {
    pub layer: u8,
    pub index_start: u32,
    pub index_count: u32,
}

#[derive(Clone, Copy, Default)]
pub struct LayerGroup {
    pub rect_start: u32,
    pub rect_count: u32,
    pub index_start: u32,
    pub index_count: u32,
    pub image_start: u32,
    pub image_count: u32,
}

pub struct PaintPass {
    descriptor_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    descriptor_pool: vk::DescriptorPool,
    quad_vertices: Buffer,
    quad_indices: Buffer,
    frames: Vec<FrameResources>,
    rects: Vec<PaintRectInstance>,
    groups: Vec<LayerGroup>,
    screen: (f32, f32),

    pub images: ImageTable,
    image_layout: vk::DescriptorSetLayout,
    image_pipeline_layout: vk::PipelineLayout,
    image_pipeline: vk::Pipeline,
    image_frames: Vec<FrameResources>,
    image_instances: Vec<PaintImageInstance>,
    text_layout: vk::DescriptorSetLayout,
    text_pipeline_layout: vk::PipelineLayout,
    text_pipeline: vk::Pipeline,
    text_frames: Vec<TextFrame>,
    text: TextDraw,
    bound_revision: u32,
}

pub fn create_pass(setup: &PassSetup<'_>) -> Result<PaintPass, Box<dyn std::error::Error>> {
    let context = setup.context;
    let descriptor_layout = create_descriptor_layout(context)?;
    let pipeline_layout = create_pipeline_layout(context, descriptor_layout)?;
    let pipeline = create_pipeline(setup, pipeline_layout)?;

    let quad_vertices = create_quad_vertices(context)?;
    let quad_indices = create_quad_indices(context)?;

    let descriptor_pool = create_descriptor_pool(context, setup.frames_in_flight)?;
    let mut frames = Vec::with_capacity(setup.frames_in_flight);
    for _ in 0..setup.frames_in_flight {
        frames.push(create_frame(
            context,
            descriptor_pool,
            descriptor_layout,
            INITIAL_RECT_CAPACITY,
        )?);
    }

    let text_layout = create_text_layout(context)?;
    let text_pipeline_layout = create_pipeline_layout(context, text_layout)?;
    let text_pipeline = create_text_pipeline(setup, text_pipeline_layout)?;
    let mut text_frames = Vec::with_capacity(setup.frames_in_flight);
    for _ in 0..setup.frames_in_flight {
        text_frames.push(create_text_frame(
            context,
            descriptor_pool,
            text_layout,
            setup.glyphs,
        )?);
    }
    let bound_revision = crate::renderer::glyph_atlas::revision(setup.glyphs);

    let images = create_image_table(context, MAX_PAINT_IMAGES, vk::ShaderStageFlags::FRAGMENT)?;
    let image_layout = create_descriptor_layout(context)?;
    let image_pipeline_layout = create_two_set_layout(context, image_layout, images.layout)?;
    let image_pipeline = create_image_pipeline(setup, image_pipeline_layout)?;
    let mut image_frames = Vec::with_capacity(setup.frames_in_flight);
    for _ in 0..setup.frames_in_flight {
        image_frames.push(create_image_frame(context, descriptor_pool, image_layout)?);
    }

    Ok(PaintPass {
        descriptor_layout,
        pipeline_layout,
        pipeline,
        descriptor_pool,
        quad_vertices,
        quad_indices,
        frames,
        rects: Vec::new(),
        groups: Vec::new(),
        screen: (1.0, 1.0),
        images,
        image_layout,
        image_pipeline_layout,
        image_pipeline,
        image_frames,
        image_instances: Vec::new(),
        text_layout,
        text_pipeline_layout,
        text_pipeline,
        text_frames,
        text: TextDraw::default(),
        bound_revision,
    })
}

pub fn hand_over(
    pass: &mut PaintPass,
    rects: Vec<PaintRectInstance>,
    rect_layers: &[u8],
    images: Vec<PaintImageInstance>,
    image_layers: &[u8],
    text: TextDraw,
    screen: (f32, f32),
) {
    pass.groups = layer_groups(rect_layers, image_layers, &text.label_ranges);
    pass.rects = rects;
    pass.image_instances = images;
    pass.text = text;
    pass.screen = screen;
}

fn layer_groups(rect_layers: &[u8], image_layers: &[u8], labels: &[LabelRange]) -> Vec<LayerGroup> {
    let mut layers: Vec<u8> = rect_layers.to_vec();
    layers.extend(image_layers.iter().copied());
    layers.extend(labels.iter().map(|label| label.layer));
    layers.sort_unstable();
    layers.dedup();

    layers
        .into_iter()
        .map(|layer| {
            let rect_start = rect_layers.partition_point(|value| *value < layer) as u32;
            let rect_end = rect_layers.partition_point(|value| *value <= layer) as u32;
            let mut index_start = u32::MAX;
            let mut index_end = 0;
            for label in labels.iter().filter(|label| label.layer == layer) {
                index_start = index_start.min(label.index_start);
                index_end = index_end.max(label.index_start + label.index_count);
            }
            let image_start = image_layers.partition_point(|value| *value < layer) as u32;
            let image_end = image_layers.partition_point(|value| *value <= layer) as u32;
            LayerGroup {
                rect_start,
                rect_count: rect_end - rect_start,
                image_start,
                image_count: image_end - image_start,
                index_start: if index_start == u32::MAX {
                    0
                } else {
                    index_start
                },
                index_count: index_end.saturating_sub(if index_start == u32::MAX {
                    0
                } else {
                    index_start
                }),
            }
        })
        .collect()
}

impl Pass for PaintPass {
    fn name(&self) -> &'static str {
        PAINT_PASS
    }

    fn draw(&mut self, record: &mut PassRecord) {
        let frame_index = record.frame_index % self.frames.len();
        if let Err(error) = self.upload(record.context, record.glyphs, frame_index) {
            tracing::error!("the paint data was not uploaded: {error}");
            return;
        }

        let device = &record.context.device;
        let command_buffer = record.command_buffer;
        let text_frame = &self.text_frames[frame_index];

        for group in &self.groups {
            if group.rect_count > 0 {
                unsafe {
                    device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline,
                    );
                    device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_layout,
                        0,
                        &[self.frames[frame_index].set],
                        &[],
                    );
                    device.cmd_bind_vertex_buffers(
                        command_buffer,
                        0,
                        &[self.quad_vertices.buffer],
                        &[0],
                    );
                    device.cmd_bind_index_buffer(
                        command_buffer,
                        self.quad_indices.buffer,
                        0,
                        vk::IndexType::UINT16,
                    );
                    device.cmd_draw_indexed(
                        command_buffer,
                        6,
                        group.rect_count,
                        0,
                        0,
                        group.rect_start,
                    );
                }
            }

            if group.image_count > 0 {
                unsafe {
                    device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.image_pipeline,
                    );
                    device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.image_pipeline_layout,
                        0,
                        &[self.image_frames[frame_index].set, self.images.set],
                        &[],
                    );
                    device.cmd_bind_vertex_buffers(
                        command_buffer,
                        0,
                        &[self.quad_vertices.buffer],
                        &[0],
                    );
                    device.cmd_bind_index_buffer(
                        command_buffer,
                        self.quad_indices.buffer,
                        0,
                        vk::IndexType::UINT16,
                    );
                    device.cmd_draw_indexed(
                        command_buffer,
                        6,
                        group.image_count,
                        0,
                        0,
                        group.image_start,
                    );
                }
            }

            if group.index_count > 0 {
                unsafe {
                    device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.text_pipeline,
                    );
                    device.cmd_bind_descriptor_sets(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.text_pipeline_layout,
                        0,
                        &[text_frame.set],
                        &[],
                    );
                    device.cmd_bind_vertex_buffers(
                        command_buffer,
                        0,
                        &[text_frame.vertices.buffer],
                        &[0],
                    );
                    device.cmd_bind_index_buffer(
                        command_buffer,
                        text_frame.indices.buffer,
                        0,
                        vk::IndexType::UINT32,
                    );
                    device.cmd_draw_indexed(
                        command_buffer,
                        group.index_count,
                        1,
                        group.index_start,
                        0,
                        0,
                    );
                }
            }
        }
    }

    fn retarget(&mut self, setup: &PassSetup<'_>) {
        let made = (|| -> Result<[vk::Pipeline; 3], Box<dyn std::error::Error>> {
            Ok([
                create_pipeline(setup, self.pipeline_layout)?,
                create_text_pipeline(setup, self.text_pipeline_layout)?,
                create_image_pipeline(setup, self.image_pipeline_layout)?,
            ])
        })();
        let [rects, text, images] = match made {
            Ok(made) => made,
            Err(error) => {
                tracing::error!("the paint was not made for the new screen format: {error}");
                return;
            }
        };
        let device = &setup.context.device;
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline(self.text_pipeline, None);
            device.destroy_pipeline(self.image_pipeline, None);
        }
        self.pipeline = rects;
        self.text_pipeline = text;
        self.image_pipeline = images;
    }

    fn destroy(&mut self, context: &Context) {
        let device = &context.device;
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline(self.text_pipeline, None);
            device.destroy_pipeline(self.image_pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_pipeline_layout(self.text_pipeline_layout, None);
            device.destroy_pipeline_layout(self.image_pipeline_layout, None);
            device.destroy_descriptor_set_layout(self.image_layout, None);
            device.destroy_descriptor_pool(self.descriptor_pool, None);
            device.destroy_descriptor_set_layout(self.descriptor_layout, None);
            device.destroy_descriptor_set_layout(self.text_layout, None);
        }
        destroy_buffer(context, &mut self.quad_vertices);
        destroy_buffer(context, &mut self.quad_indices);
        for frame in &mut self.frames {
            destroy_buffer(context, &mut frame.globals);
            destroy_buffer(context, &mut frame.instances);
        }
        for frame in &mut self.text_frames {
            destroy_buffer(context, &mut frame.globals);
            destroy_buffer(context, &mut frame.instances);
            destroy_buffer(context, &mut frame.character_colors);
            destroy_buffer(context, &mut frame.character_backgrounds);
            destroy_buffer(context, &mut frame.vertices);
            destroy_buffer(context, &mut frame.indices);
        }
        for frame in &mut self.image_frames {
            destroy_buffer(context, &mut frame.globals);
            destroy_buffer(context, &mut frame.instances);
        }
        destroy_image_table(context, &mut self.images);
    }
}

impl PaintPass {
    fn upload(
        &mut self,
        context: &Context,
        glyphs: &GlyphAtlas,
        frame_index: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.frames[frame_index].capacity < self.rects.len() {
            let capacity = self.rects.len().next_power_of_two();
            let instances = create_instance_buffer(context, capacity)?;
            let frame = &mut self.frames[frame_index];
            destroy_buffer(context, &mut frame.instances);
            frame.instances = instances;
            frame.capacity = capacity;
            bind_frame(context, frame, self.descriptor_layout);
        }

        let projection = nalgebra_glm::ortho(0.0, self.screen.0, self.screen.1, 0.0, -1.0, 1.0);
        let mut columns = [[0.0f32; 4]; 4];
        for column in 0..4 {
            for row in 0..4 {
                columns[column][row] = projection[(row, column)];
            }
        }
        let frame = &self.frames[frame_index];
        write_buffer(
            &frame.globals,
            std::slice::from_ref(&GlobalUniforms {
                projection: columns,
            }),
        );
        write_buffer(&frame.instances, &self.rects);

        if self.image_frames[frame_index].capacity < self.image_instances.len() {
            let capacity = self.image_instances.len().next_power_of_two();
            let instances = create_storage(
                context,
                "paint image instances",
                (capacity * std::mem::size_of::<PaintImageInstance>()) as u64,
            )?;
            let frame = &mut self.image_frames[frame_index];
            destroy_buffer(context, &mut frame.instances);
            frame.instances = instances;
            frame.capacity = capacity;
            bind_frame(context, frame, self.image_layout);
        }
        let frame = &self.image_frames[frame_index];
        write_buffer(
            &frame.globals,
            std::slice::from_ref(&GlobalUniforms {
                projection: columns,
            }),
        );
        write_buffer(&frame.instances, &self.image_instances);

        self.upload_text(context, glyphs, frame_index, columns)?;
        Ok(())
    }

    fn upload_text(
        &mut self,
        context: &Context,
        glyphs: &GlyphAtlas,
        frame_index: usize,
        projection: [[f32; 4]; 4],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let revision = crate::renderer::glyph_atlas::revision(glyphs);
        let atlas_changed = revision != self.bound_revision;
        if atlas_changed {
            for frame in &self.text_frames {
                bind_text_frame(context, frame, glyphs);
            }
            self.bound_revision = revision;
        }
        if self.text.indices.is_empty() {
            return Ok(());
        }

        let mut rebind = false;
        {
            let frame = &mut self.text_frames[frame_index];
            if frame.instance_capacity < self.text.instances.len() {
                let capacity = self.text.instances.len().next_power_of_two();
                let buffer = create_storage(
                    context,
                    "paint text instances",
                    (capacity * std::mem::size_of::<TextInstanceData>()) as u64,
                )?;
                destroy_buffer(context, &mut frame.instances);
                frame.instances = buffer;
                frame.instance_capacity = capacity;
                rebind = true;
            }
            let characters = self
                .text
                .character_colors
                .len()
                .max(self.text.character_backgrounds.len())
                .max(1);
            if frame.character_capacity < characters {
                let capacity = characters.next_power_of_two();
                let colors =
                    create_storage(context, "paint character colors", (capacity * 16) as u64)?;
                let backgrounds = create_storage(
                    context,
                    "paint character backgrounds",
                    (capacity * 16) as u64,
                )?;
                destroy_buffer(context, &mut frame.character_colors);
                destroy_buffer(context, &mut frame.character_backgrounds);
                frame.character_colors = colors;
                frame.character_backgrounds = backgrounds;
                frame.character_capacity = capacity;
                rebind = true;
            }
            if frame.vertex_capacity < self.text.vertices.len() {
                let capacity = self.text.vertices.len().next_power_of_two();
                let buffer = create_buffer(
                    context,
                    "paint text vertices",
                    (capacity * std::mem::size_of::<UiTextVertex>()) as u64,
                    vk::BufferUsageFlags::VERTEX_BUFFER,
                    MemoryLocation::CpuToGpu,
                )?;
                destroy_buffer(context, &mut frame.vertices);
                frame.vertices = buffer;
                frame.vertex_capacity = capacity;
            }
            if frame.index_capacity < self.text.indices.len() {
                let capacity = self.text.indices.len().next_power_of_two();
                let buffer = create_buffer(
                    context,
                    "paint text indices",
                    (capacity * 4) as u64,
                    vk::BufferUsageFlags::INDEX_BUFFER,
                    MemoryLocation::CpuToGpu,
                )?;
                destroy_buffer(context, &mut frame.indices);
                frame.indices = buffer;
                frame.index_capacity = capacity;
            }
        }

        if rebind {
            bind_text_frame(context, &self.text_frames[frame_index], glyphs);
        }

        let frame = &self.text_frames[frame_index];
        write_buffer(
            &frame.globals,
            std::slice::from_ref(&GlobalUniforms { projection }),
        );
        write_buffer(&frame.instances, &self.text.instances);
        write_buffer(&frame.character_colors, &self.text.character_colors);
        write_buffer(
            &frame.character_backgrounds,
            &self.text.character_backgrounds,
        );
        write_buffer(&frame.vertices, &self.text.vertices);
        write_buffer(&frame.indices, &self.text.indices);
        Ok(())
    }
}

fn create_frame(
    context: &Context,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    capacity: usize,
) -> Result<FrameResources, Box<dyn std::error::Error>> {
    let globals = create_buffer(
        context,
        "paint globals",
        std::mem::size_of::<GlobalUniforms>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;
    let instances = create_instance_buffer(context, capacity)?;

    let layouts = [layout];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let set = unsafe { context.device.allocate_descriptor_sets(&allocate_info)?[0] };

    let mut frame = FrameResources {
        globals,
        instances,
        set,
        capacity,
    };
    bind_frame(context, &mut frame, layout);
    Ok(frame)
}

fn bind_frame(context: &Context, frame: &mut FrameResources, _layout: vk::DescriptorSetLayout) {
    let globals = vk::DescriptorBufferInfo::default()
        .buffer(frame.globals.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let instances = vk::DescriptorBufferInfo::default()
        .buffer(frame.instances.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let writes = [
        vk::WriteDescriptorSet::default()
            .dst_set(frame.set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&globals)),
        vk::WriteDescriptorSet::default()
            .dst_set(frame.set)
            .dst_binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&instances)),
    ];
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

fn create_instance_buffer(
    context: &Context,
    capacity: usize,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        "paint rect instances",
        (capacity * std::mem::size_of::<PaintRectInstance>()) as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        MemoryLocation::CpuToGpu,
    )
}

fn create_quad_vertices(context: &Context) -> Result<Buffer, Box<dyn std::error::Error>> {
    let vertices = [
        QuadVertex {
            position: [0.0, 0.0],
        },
        QuadVertex {
            position: [1.0, 0.0],
        },
        QuadVertex {
            position: [1.0, 1.0],
        },
        QuadVertex {
            position: [0.0, 1.0],
        },
    ];
    let buffer = create_buffer(
        context,
        "paint quad vertices",
        std::mem::size_of_val(&vertices) as u64,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;
    write_buffer(&buffer, &vertices);
    Ok(buffer)
}

fn create_quad_indices(context: &Context) -> Result<Buffer, Box<dyn std::error::Error>> {
    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
    let buffer = create_buffer(
        context,
        "paint quad indices",
        std::mem::size_of_val(&indices) as u64,
        vk::BufferUsageFlags::INDEX_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;
    write_buffer(&buffer, &indices);
    Ok(buffer)
}

fn create_descriptor_layout(
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
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT),
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

fn create_descriptor_pool(
    context: &Context,
    frames: usize,
) -> Result<vk::DescriptorPool, Box<dyn std::error::Error>> {
    let count = frames as u32;
    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(count * 3),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(count * 5),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(count),
    ];
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&sizes)
        .max_sets(count * 3);
    Ok(unsafe { context.device.create_descriptor_pool(&create_info, None)? })
}

fn create_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let vertex = create_shader_module(context, include_bytes!("shaders/paint_rect.vert.spv"))?;
    let fragment = create_shader_module(context, include_bytes!("shaders/paint_rect.frag.spv"))?;

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
        .stride(std::mem::size_of::<QuadVertex>() as u32)
        .input_rate(vk::VertexInputRate::VERTEX);
    let attribute = vk::VertexInputAttributeDescription::default()
        .location(0)
        .binding(0)
        .format(vk::Format::R32G32_SFLOAT)
        .offset(0);
    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(std::slice::from_ref(&binding))
        .vertex_attribute_descriptions(std::slice::from_ref(&attribute));

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
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
    let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(std::slice::from_ref(&blend_attachment));

    let color_formats = [setup.color_format];
    let mut rendering = vk::PipelineRenderingCreateInfo::default()
        .color_attachment_formats(&color_formats)
        .depth_attachment_format(setup.depth_format);

    let create_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
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

fn create_storage(
    context: &Context,
    name: &str,
    size: u64,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    create_buffer(
        context,
        name,
        size.max(16),
        vk::BufferUsageFlags::STORAGE_BUFFER,
        MemoryLocation::CpuToGpu,
    )
}

fn create_text_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let stages = vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT;
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
            .stage_flags(stages),
        vk::DescriptorSetLayoutBinding::default()
            .binding(2)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(3)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
        vk::DescriptorSetLayoutBinding::default()
            .binding(4)
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

fn create_text_frame(
    context: &Context,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    atlas: &GlyphAtlas,
) -> Result<TextFrame, Box<dyn std::error::Error>> {
    let globals = create_buffer(
        context,
        "paint text globals",
        std::mem::size_of::<GlobalUniforms>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;
    let instances = create_storage(context, "paint text instances", 1024)?;
    let character_colors = create_storage(context, "paint character colors", 1024)?;
    let character_backgrounds = create_storage(context, "paint character backgrounds", 1024)?;
    let vertices = create_buffer(
        context,
        "paint text vertices",
        (1024 * std::mem::size_of::<UiTextVertex>()) as u64,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;
    let indices = create_buffer(
        context,
        "paint text indices",
        1024 * 4,
        vk::BufferUsageFlags::INDEX_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;

    let layouts = [layout];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let set = unsafe { context.device.allocate_descriptor_sets(&allocate_info)?[0] };

    let frame = TextFrame {
        globals,
        instances,
        character_colors,
        character_backgrounds,
        vertices,
        indices,
        set,
        instance_capacity: 1024 / std::mem::size_of::<TextInstanceData>(),
        vertex_capacity: 1024,
        index_capacity: 1024,
        character_capacity: 64,
    };
    bind_text_frame(context, &frame, atlas);
    Ok(frame)
}

fn bind_text_frame(context: &Context, frame: &TextFrame, atlas: &GlyphAtlas) {
    let globals = vk::DescriptorBufferInfo::default()
        .buffer(frame.globals.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let instances = vk::DescriptorBufferInfo::default()
        .buffer(frame.instances.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let colors = vk::DescriptorBufferInfo::default()
        .buffer(frame.character_colors.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let backgrounds = vk::DescriptorBufferInfo::default()
        .buffer(frame.character_backgrounds.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let font = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(atlas.view)
        .sampler(atlas.sampler);
    let writes = [
        vk::WriteDescriptorSet::default()
            .dst_set(frame.set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&globals)),
        vk::WriteDescriptorSet::default()
            .dst_set(frame.set)
            .dst_binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&instances)),
        vk::WriteDescriptorSet::default()
            .dst_set(frame.set)
            .dst_binding(2)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&colors)),
        vk::WriteDescriptorSet::default()
            .dst_set(frame.set)
            .dst_binding(3)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(std::slice::from_ref(&backgrounds)),
        vk::WriteDescriptorSet::default()
            .dst_set(frame.set)
            .dst_binding(4)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&font)),
    ];
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

fn create_text_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let vertex = create_shader_module(context, include_bytes!("shaders/paint_text.vert.spv"))?;
    let fragment = create_shader_module(context, include_bytes!("shaders/paint_text.frag.spv"))?;

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
        .stride(std::mem::size_of::<UiTextVertex>() as u32)
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
        vk::VertexInputAttributeDescription::default()
            .location(4)
            .binding(0)
            .format(vk::Format::R32_UINT)
            .offset(28),
    ];
    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(std::slice::from_ref(&binding))
        .vertex_attribute_descriptions(&attributes);

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
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
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .alpha_blend_op(vk::BlendOp::ADD);
    let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(std::slice::from_ref(&blend_attachment));

    let color_formats = [setup.color_format];
    let mut rendering = vk::PipelineRenderingCreateInfo::default()
        .color_attachment_formats(&color_formats)
        .depth_attachment_format(setup.depth_format);

    let create_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
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

fn create_two_set_layout(
    context: &Context,
    first: vk::DescriptorSetLayout,
    second: vk::DescriptorSetLayout,
) -> Result<vk::PipelineLayout, Box<dyn std::error::Error>> {
    let layouts = [first, second];
    let create_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
    Ok(unsafe { context.device.create_pipeline_layout(&create_info, None)? })
}

fn create_image_frame(
    context: &Context,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
) -> Result<FrameResources, Box<dyn std::error::Error>> {
    let globals = create_buffer(
        context,
        "paint image globals",
        std::mem::size_of::<GlobalUniforms>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;
    let capacity = 256;
    let instances = create_storage(
        context,
        "paint image instances",
        (capacity * std::mem::size_of::<PaintImageInstance>()) as u64,
    )?;

    let layouts = [layout];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let set = unsafe { context.device.allocate_descriptor_sets(&allocate_info)?[0] };

    let mut frame = FrameResources {
        globals,
        instances,
        set,
        capacity,
    };
    bind_frame(context, &mut frame, layout);
    Ok(frame)
}

fn create_image_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let vertex = create_shader_module(context, include_bytes!("shaders/paint_image.vert.spv"))?;
    let fragment = create_shader_module(context, include_bytes!("shaders/paint_image.frag.spv"))?;

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
        .stride(std::mem::size_of::<QuadVertex>() as u32)
        .input_rate(vk::VertexInputRate::VERTEX);
    let attribute = vk::VertexInputAttributeDescription::default()
        .location(0)
        .binding(0)
        .format(vk::Format::R32G32_SFLOAT)
        .offset(0);
    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(std::slice::from_ref(&binding))
        .vertex_attribute_descriptions(std::slice::from_ref(&attribute));

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
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
    let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(std::slice::from_ref(&blend_attachment));

    let color_formats = [setup.color_format];
    let mut rendering = vk::PipelineRenderingCreateInfo::default()
        .color_attachment_formats(&color_formats)
        .depth_attachment_format(setup.depth_format);

    let create_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
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
