use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};
use nalgebra_glm::Mat4;
use nightshade_sprite::contract::SpriteFilter;
use nightshade_sprite::contract::render_data::{
    SpriteAtlasSpec, SpriteAtlasUpload, SpriteInstance,
};

use crate::renderer::context::Context;
use crate::renderer::memory::{
    Buffer, begin_one_shot, create_buffer, destroy_buffer, end_one_shot, release, retire_buffer,
    transition, write_buffer,
};
use crate::renderer::pass::{Pass, PassRecord, PassSetup, ScopeKind, create_shader_module};

pub const SPRITES_PASS: &str = "sprites";

const MAX_SPRITES: usize = 1 << 15;
const MAX_ATLASES: usize = 64;
const VERTICES_PER_SPRITE: u32 = 6;
const FLAG_BILLBOARD: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuSprite {
    model: [[f32; 4]; 4],
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    size: [f32; 2],
    anchor: [f32; 2],
    color: [f32; 4],
    flags: u32,
    lean: f32,
    padding: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Camera {
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    camera_right: [f32; 4],
    camera_up: [f32; 4],
}

pub struct SpriteFrameData {
    pub instances: Vec<SpriteInstance>,
    pub atlases: Vec<SpriteAtlasSpec>,
    pub uploads: Vec<SpriteAtlasUpload>,
    pub view: Mat4,
    pub projection: Mat4,
    pub camera_right: [f32; 4],
    pub camera_up: [f32; 4],
}

impl Default for SpriteFrameData {
    fn default() -> Self {
        Self {
            instances: Vec::new(),
            atlases: Vec::new(),
            uploads: Vec::new(),
            view: Mat4::identity(),
            projection: Mat4::identity(),
            camera_right: [1.0, 0.0, 0.0, 0.0],
            camera_up: [0.0, 1.0, 0.0, 0.0],
        }
    }
}

struct Atlas {
    size: u32,
    filter: SpriteFilter,
    image: vk::Image,
    view: vk::ImageView,
    allocation: Option<Allocation>,
}

struct Batch {
    atlas: usize,
    first: u32,
    count: u32,
}

pub struct SpritesPass {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_layout: vk::DescriptorSetLayout,
    atlas_layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    set: vk::DescriptorSet,
    atlas_sets: Vec<vk::DescriptorSet>,
    camera: Buffer,
    sprites: Buffer,
    nearest: vk::Sampler,
    linear: vk::Sampler,
    atlases: Vec<Option<Atlas>>,
    pending: Vec<SpriteAtlasUpload>,
    data: SpriteFrameData,
    gpu_sprites: Vec<GpuSprite>,
    batches: Vec<Batch>,
}

pub fn create_pass(setup: &PassSetup<'_>) -> Result<SpritesPass, Box<dyn std::error::Error>> {
    let context = setup.context;
    let descriptor_layout = create_descriptor_layout(context)?;
    let atlas_layout = create_atlas_layout(context)?;
    let pool = create_descriptor_pool(context)?;
    let set = allocate_sets(context, pool, &[descriptor_layout])?[0];
    let atlas_sets = allocate_sets(context, pool, &vec![atlas_layout; MAX_ATLASES])?;
    let nearest = create_sampler(context, vk::Filter::NEAREST)?;
    let linear = create_sampler(context, vk::Filter::LINEAR)?;

    let camera = create_buffer(
        context,
        "sprite camera",
        std::mem::size_of::<Camera>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;
    let sprites = create_buffer(
        context,
        "sprites",
        (MAX_SPRITES * std::mem::size_of::<GpuSprite>()) as u64,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        MemoryLocation::CpuToGpu,
    )?;

    let layouts = [descriptor_layout, atlas_layout];
    let layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
    let pipeline_layout = unsafe { context.device.create_pipeline_layout(&layout_info, None)? };
    let pipeline = create_pipeline(setup, pipeline_layout)?;

    let pass = SpritesPass {
        pipeline,
        pipeline_layout,
        descriptor_layout,
        atlas_layout,
        pool,
        set,
        atlas_sets,
        camera,
        sprites,
        nearest,
        linear,
        atlases: Vec::new(),
        pending: Vec::new(),
        data: SpriteFrameData::default(),
        gpu_sprites: Vec::new(),
        batches: Vec::new(),
    };
    write_set(context, &pass);
    Ok(pass)
}

pub fn hand_over(pass: &mut SpritesPass, mut data: SpriteFrameData) {
    pass.pending.append(&mut data.uploads);
    pass.data = data;
}

impl Pass for SpritesPass {
    fn name(&self) -> &'static str {
        SPRITES_PASS
    }

    fn compute(&mut self, record: &mut PassRecord) {
        let context = record.context;
        self.ensure_atlases(context);
        self.apply_uploads(context);
        self.build_batches();
        if self.gpu_sprites.is_empty() {
            return;
        }
        write_buffer(
            &self.camera,
            std::slice::from_ref(&Camera {
                view: self.data.view.into(),
                projection: self.data.projection.into(),
                camera_right: self.data.camera_right,
                camera_up: self.data.camera_up,
            }),
        );
        write_buffer(&self.sprites, &self.gpu_sprites);
    }

    fn draw(&mut self, record: &mut PassRecord) {
        if record.scope != ScopeKind::OverSolids || self.batches.is_empty() {
            return;
        }
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
                &[self.set],
                &[],
            );
            for batch in &self.batches {
                device.cmd_bind_descriptor_sets(
                    record.command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline_layout,
                    1,
                    &[self.atlas_sets[batch.atlas]],
                    &[],
                );
                device.cmd_draw(
                    record.command_buffer,
                    VERTICES_PER_SPRITE,
                    batch.count,
                    0,
                    batch.first,
                );
            }
        }
    }

    fn destroy(&mut self, context: &Context) {
        for atlas in self.atlases.iter_mut().filter_map(Option::take) {
            destroy_atlas(context, atlas);
        }
        unsafe {
            context.device.destroy_pipeline(self.pipeline, None);
            context
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            context.device.destroy_descriptor_pool(self.pool, None);
            context
                .device
                .destroy_descriptor_set_layout(self.descriptor_layout, None);
            context
                .device
                .destroy_descriptor_set_layout(self.atlas_layout, None);
            context.device.destroy_sampler(self.nearest, None);
            context.device.destroy_sampler(self.linear, None);
        }
        destroy_buffer(context, &mut self.camera);
        destroy_buffer(context, &mut self.sprites);
    }
}

impl SpritesPass {
    fn ensure_atlases(&mut self, context: &Context) {
        let specs = std::mem::take(&mut self.data.atlases);
        for spec in &specs {
            let index = spec.index as usize;
            if index >= MAX_ATLASES {
                tracing::error!("sprite atlas {index} is past the {MAX_ATLASES} the pass holds");
                continue;
            }
            if index >= self.atlases.len() {
                self.atlases.resize_with(index + 1, || None);
            }
            let stale = self.atlases[index]
                .as_ref()
                .is_some_and(|atlas| atlas.size != spec.size || atlas.filter != spec.filter);
            if self.atlases[index].is_some() && !stale {
                continue;
            }
            if let Some(old) = self.atlases[index].take() {
                if let Err(error) = unsafe { context.device.device_wait_idle() } {
                    tracing::error!(
                        "the device did not settle before an atlas was replaced: {error}"
                    );
                }
                destroy_atlas(context, old);
            }
            let size = spec.size.max(1);
            let atlas = match create_atlas(context, size, spec.filter) {
                Ok(atlas) => atlas,
                Err(error) => {
                    tracing::error!("sprite atlas {index} was not made: {error}");
                    continue;
                }
            };
            let sampler = match spec.filter {
                SpriteFilter::Nearest => self.nearest,
                SpriteFilter::Linear => self.linear,
            };
            write_atlas_set(context, self.atlas_sets[index], atlas.view, sampler);
            self.atlases[index] = Some(atlas);
        }
        self.data.atlases = specs;
    }

    fn apply_uploads(&mut self, context: &Context) {
        let uploads = std::mem::take(&mut self.pending);
        for upload in uploads {
            let Some(atlas) = self
                .atlases
                .get(upload.atlas as usize)
                .and_then(|slot| slot.as_ref())
            else {
                self.pending.push(upload);
                continue;
            };
            if upload.width == 0
                || upload.height == 0
                || upload.x + upload.width > atlas.size
                || upload.y + upload.height > atlas.size
                || upload.rgba.len() != (upload.width * upload.height * 4) as usize
            {
                continue;
            }
            if let Err(error) = upload_pixels(context, atlas, &upload) {
                tracing::error!("a sprite atlas was not uploaded: {error}");
            }
        }
    }

    fn build_batches(&mut self) {
        self.gpu_sprites.clear();
        self.batches.clear();

        for index in nightshade_sprite::contract::render_data::sprite_draw_order(
            &self.data.instances,
            &self.data.view,
        ) {
            if self.gpu_sprites.len() >= MAX_SPRITES {
                break;
            }
            let instance = &self.data.instances[index];
            let atlas = instance.atlas as usize;
            if self.atlases.get(atlas).is_none_or(|slot| slot.is_none()) {
                continue;
            }
            let slot = self.gpu_sprites.len() as u32;
            self.gpu_sprites.push(GpuSprite {
                model: instance.transform.into(),
                uv_min: [instance.uv_min.x, instance.uv_min.y],
                uv_max: [instance.uv_max.x, instance.uv_max.y],
                size: [instance.size.x, instance.size.y],
                anchor: [instance.anchor.x, instance.anchor.y],
                color: instance.color,
                flags: if instance.billboard {
                    FLAG_BILLBOARD
                } else {
                    0
                },
                lean: instance.lean,
                padding: [0; 2],
            });
            match self.batches.last_mut() {
                Some(batch) if batch.atlas == atlas => batch.count += 1,
                _ => self.batches.push(Batch {
                    atlas,
                    first: slot,
                    count: 1,
                }),
            }
        }
    }
}

fn write_set(context: &Context, pass: &SpritesPass) {
    let camera = vk::DescriptorBufferInfo::default()
        .buffer(pass.camera.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
    let sprites = vk::DescriptorBufferInfo::default()
        .buffer(pass.sprites.buffer)
        .offset(0)
        .range(vk::WHOLE_SIZE);
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
            .buffer_info(std::slice::from_ref(&sprites)),
    ];
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

fn write_atlas_set(
    context: &Context,
    set: vk::DescriptorSet,
    view: vk::ImageView,
    sampler: vk::Sampler,
) {
    let image = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(view)
        .sampler(sampler);
    let write = vk::WriteDescriptorSet::default()
        .dst_set(set)
        .dst_binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(std::slice::from_ref(&image));
    unsafe {
        context
            .device
            .update_descriptor_sets(std::slice::from_ref(&write), &[])
    };
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
            .stage_flags(vk::ShaderStageFlags::VERTEX),
    ];
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

fn create_atlas_layout(
    context: &Context,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let bindings = [vk::DescriptorSetLayoutBinding::default()
        .binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
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
            .descriptor_count(1),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(MAX_ATLASES as u32),
    ];
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(1 + MAX_ATLASES as u32)
        .pool_sizes(&sizes);
    Ok(unsafe { context.device.create_descriptor_pool(&create_info, None)? })
}

fn allocate_sets(
    context: &Context,
    pool: vk::DescriptorPool,
    layouts: &[vk::DescriptorSetLayout],
) -> Result<Vec<vk::DescriptorSet>, Box<dyn std::error::Error>> {
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(layouts);
    Ok(unsafe { context.device.allocate_descriptor_sets(&allocate_info)? })
}

fn create_sampler(
    context: &Context,
    filter: vk::Filter,
) -> Result<vk::Sampler, Box<dyn std::error::Error>> {
    let create_info = vk::SamplerCreateInfo::default()
        .mag_filter(filter)
        .min_filter(filter)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE);
    Ok(unsafe { context.device.create_sampler(&create_info, None)? })
}

fn create_atlas(
    context: &Context,
    size: u32,
    filter: SpriteFilter,
) -> Result<Atlas, Box<dyn std::error::Error>> {
    let create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(vk::Format::R8G8B8A8_SRGB)
        .extent(vk::Extent3D {
            width: size,
            height: size,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);
    let image = unsafe { context.device.create_image(&create_info, None)? };
    let requirements = unsafe { context.device.get_image_memory_requirements(image) };
    let allocation = context
        .allocator()
        .lock()
        .map_err(|error| error.to_string())?
        .allocate(&AllocationCreateDesc {
            name: "sprite atlas",
            requirements,
            location: MemoryLocation::GpuOnly,
            linear: false,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;
    unsafe {
        context
            .device
            .bind_image_memory(image, allocation.memory(), allocation.offset())?;
    }

    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(vk::Format::R8G8B8A8_SRGB)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };

    let command_buffer = begin_one_shot(context)?;
    transition(
        context,
        command_buffer,
        image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    end_one_shot(context, command_buffer)?;

    Ok(Atlas {
        size,
        filter,
        image,
        view,
        allocation: Some(allocation),
    })
}

fn destroy_atlas(context: &Context, mut atlas: Atlas) {
    unsafe {
        context.device.destroy_image_view(atlas.view, None);
        context.device.destroy_image(atlas.image, None);
    }
    release(context, atlas.allocation.take());
}

fn upload_pixels(
    context: &Context,
    atlas: &Atlas,
    upload: &SpriteAtlasUpload,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut staging = create_buffer(
        context,
        "sprite atlas staging",
        upload.rgba.len() as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )?;
    write_buffer(&staging, &upload.rgba);

    let command_buffer = begin_one_shot(context)?;
    transition(
        context,
        command_buffer,
        atlas.image,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );
    let region = vk::BufferImageCopy::default()
        .image_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        })
        .image_offset(vk::Offset3D {
            x: upload.x as i32,
            y: upload.y as i32,
            z: 0,
        })
        .image_extent(vk::Extent3D {
            width: upload.width,
            height: upload.height,
            depth: 1,
        });
    unsafe {
        context.device.cmd_copy_buffer_to_image(
            command_buffer,
            staging.buffer,
            atlas.image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            std::slice::from_ref(&region),
        )
    };
    transition(
        context,
        command_buffer,
        atlas.image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    end_one_shot(context, command_buffer)?;
    retire_buffer(context, &mut staging);
    Ok(())
}

fn create_pipeline(
    setup: &PassSetup<'_>,
    layout: vk::PipelineLayout,
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let context = setup.context;
    let vertex = create_shader_module(context, include_bytes!("shaders/sprite.vert.spv"))?;
    let fragment = create_shader_module(context, include_bytes!("shaders/sprite.frag.spv"))?;
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
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .alpha_blend_op(vk::BlendOp::ADD);
    let untouched = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::empty());
    let attachments = [blend_attachment, untouched, untouched];
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
