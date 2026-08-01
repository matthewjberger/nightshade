use crate::renderer::context::Context;
use crate::renderer::memory::{Image, create_chain_image, destroy_image};
use crate::renderer::pass::create_shader_module;
use ash::vk;

const GROUP_SIZE: u32 = 8;

pub const FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct LevelPush {
    destination_size: [u32; 2],
    source_size: [u32; 2],
}

pub struct Grab {
    image: Image,
    levels: Vec<vk::ImageView>,
    sampler: vk::Sampler,
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    sets: Vec<vk::DescriptorSet>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    extent: vk::Extent2D,
}

fn level_count(extent: vk::Extent2D) -> u32 {
    let widest = extent.width.max(extent.height).max(1);
    32 - widest.leading_zeros()
}

fn level_extent(extent: vk::Extent2D, level: u32) -> vk::Extent2D {
    vk::Extent2D {
        width: (extent.width >> level).max(1),
        height: (extent.height >> level).max(1),
    }
}

pub fn create(
    context: &Context,
    extent: vk::Extent2D,
    scene: vk::ImageView,
) -> Result<Grab, Box<dyn std::error::Error>> {
    let count = level_count(extent);
    let image = create_chain_image(context, extent, count, FORMAT, "scene grab")?;

    let mut levels = Vec::with_capacity(count as usize);
    for level in 0..count {
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image.image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(FORMAT)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: level,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        levels.push(unsafe { context.device.create_image_view(&view_info, None)? });
    }

    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .min_lod(0.0)
        .max_lod(count as f32);
    let sampler = unsafe { context.device.create_sampler(&sampler_info, None)? };

    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE),
    ];
    let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    let layout = unsafe {
        context
            .device
            .create_descriptor_set_layout(&layout_info, None)?
    };

    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(count),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_IMAGE)
            .descriptor_count(count),
    ];
    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&sizes)
        .max_sets(count);
    let pool = unsafe { context.device.create_descriptor_pool(&pool_info, None)? };

    let layouts = vec![layout; count as usize];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let sets = unsafe { context.device.allocate_descriptor_sets(&allocate_info)? };

    let range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::COMPUTE)
        .offset(0)
        .size(std::mem::size_of::<LevelPush>() as u32);
    let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(std::slice::from_ref(&layout))
        .push_constant_ranges(std::slice::from_ref(&range));
    let pipeline_layout = unsafe {
        context
            .device
            .create_pipeline_layout(&pipeline_layout_info, None)?
    };
    let pipeline = create_compute(
        context,
        pipeline_layout,
        include_bytes!("shaders/grab_copy.comp.spv"),
    )?;

    let grab = Grab {
        image,
        levels,
        sampler,
        layout,
        pool,
        sets,
        pipeline_layout,
        pipeline,
        extent,
    };
    write_sets(context, &grab, scene);
    Ok(grab)
}

fn create_compute(
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
    let pipelines = unsafe {
        context.device.create_compute_pipelines(
            context.pipeline_cache,
            std::slice::from_ref(&create_info),
            None,
        )
    };
    unsafe { context.device.destroy_shader_module(module, None) };
    match pipelines {
        Ok(pipelines) => Ok(pipelines[0]),
        Err((_, error)) => Err(error.into()),
    }
}

fn write_sets(context: &Context, grab: &Grab, scene: vk::ImageView) {
    let sources: Vec<vk::DescriptorImageInfo> = (0..grab.levels.len())
        .map(|level| {
            let (view, layout) = if level == 0 {
                (scene, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            } else {
                (grab.levels[level - 1], vk::ImageLayout::GENERAL)
            };
            vk::DescriptorImageInfo::default()
                .image_view(view)
                .sampler(grab.sampler)
                .image_layout(layout)
        })
        .collect();
    let destinations: Vec<vk::DescriptorImageInfo> = grab
        .levels
        .iter()
        .map(|view| {
            vk::DescriptorImageInfo::default()
                .image_view(*view)
                .image_layout(vk::ImageLayout::GENERAL)
        })
        .collect();

    let mut writes = Vec::with_capacity(grab.sets.len() * 2);
    for (level, set) in grab.sets.iter().enumerate() {
        writes.push(
            vk::WriteDescriptorSet::default()
                .dst_set(*set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&sources[level])),
        );
        writes.push(
            vk::WriteDescriptorSet::default()
                .dst_set(*set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(&destinations[level])),
        );
    }
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

pub fn view(grab: &Grab) -> vk::ImageView {
    grab.image.view
}

pub fn image(grab: &Grab) -> vk::Image {
    grab.image.image
}

pub fn sampler(grab: &Grab) -> vk::Sampler {
    grab.sampler
}

pub fn build(grab: &Grab, context: &Context, command_buffer: vk::CommandBuffer) {
    let device = &context.device;
    let count = grab.levels.len() as u32;
    unsafe {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            grab.pipeline,
        );
    }
    for level in 0..count {
        if level > 0 {
            between_levels(device, command_buffer, grab.image.image, count);
        }
        let destination = level_extent(grab.extent, level);
        let source = if level == 0 {
            grab.extent
        } else {
            level_extent(grab.extent, level - 1)
        };
        let push = LevelPush {
            destination_size: [destination.width, destination.height],
            source_size: [source.width, source.height],
        };
        unsafe {
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                grab.pipeline_layout,
                0,
                &[grab.sets[level as usize]],
                &[],
            );
            device.cmd_push_constants(
                command_buffer,
                grab.pipeline_layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                bytemuck::bytes_of(&push),
            );
            device.cmd_dispatch(
                command_buffer,
                destination.width.div_ceil(GROUP_SIZE),
                destination.height.div_ceil(GROUP_SIZE),
                1,
            );
        }
    }
}

fn between_levels(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    levels: u32,
) {
    let barrier = vk::ImageMemoryBarrier2::default()
        .image(image)
        .old_layout(vk::ImageLayout::GENERAL)
        .new_layout(vk::ImageLayout::GENERAL)
        .src_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE)
        .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
        .src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
        .dst_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: levels,
            base_array_layer: 0,
            layer_count: 1,
        });
    let dependency =
        vk::DependencyInfo::default().image_memory_barriers(std::slice::from_ref(&barrier));
    unsafe { device.cmd_pipeline_barrier2(command_buffer, &dependency) };
}

pub fn destroy(grab: &mut Grab, context: &Context) {
    unsafe {
        context.device.destroy_pipeline(grab.pipeline, None);
        context
            .device
            .destroy_pipeline_layout(grab.pipeline_layout, None);
        context.device.destroy_descriptor_pool(grab.pool, None);
        context
            .device
            .destroy_descriptor_set_layout(grab.layout, None);
        context.device.destroy_sampler(grab.sampler, None);
        for view in grab.levels.drain(..) {
            context.device.destroy_image_view(view, None);
        }
    }
    destroy_image(context, &mut grab.image);
}
