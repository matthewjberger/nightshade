use crate::renderer::context::Context;
use crate::renderer::memory::{Image, create_pyramid_image, destroy_image};
use crate::renderer::pass::create_shader_module;
use ash::vk;

const GROUP_SIZE: u32 = 8;

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct LevelPush {
    destination_size: [u32; 2],
    source_size: [u32; 2],
}

pub struct DepthPyramid {
    image: Image,
    levels: Vec<vk::ImageView>,
    sampler: vk::Sampler,
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    sets: Vec<vk::DescriptorSet>,
    pipeline_layout: vk::PipelineLayout,
    depth_pipeline: vk::Pipeline,
    reduce_pipeline: vk::Pipeline,
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
    depth: vk::ImageView,
) -> Result<DepthPyramid, Box<dyn std::error::Error>> {
    let count = level_count(extent);
    let image = create_pyramid_image(context, extent, count, "depth pyramid")?;

    let mut levels = Vec::with_capacity(count as usize);
    for level in 0..count {
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image.image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(image.format)
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
        .mag_filter(vk::Filter::NEAREST)
        .min_filter(vk::Filter::NEAREST)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
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
        vk::DescriptorSetLayoutBinding::default()
            .binding(2)
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
            .descriptor_count(count * 2),
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

    let depth_pipeline = create_compute(
        context,
        pipeline_layout,
        include_bytes!("shaders/hiz_depth.comp.spv"),
    )?;
    let reduce_pipeline = create_compute(
        context,
        pipeline_layout,
        include_bytes!("shaders/hiz_reduce.comp.spv"),
    )?;

    let pyramid = DepthPyramid {
        image,
        levels,
        sampler,
        layout,
        pool,
        sets,
        pipeline_layout,
        depth_pipeline,
        reduce_pipeline,
        extent,
    };
    write_sets(context, &pyramid, depth);
    Ok(pyramid)
}

fn create_compute(
    context: &Context,
    layout: vk::PipelineLayout,
    code: &[u8],
) -> Result<vk::Pipeline, Box<dyn std::error::Error>> {
    let module = create_shader_module(context, code)?;
    let entry = c"main";
    let stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::COMPUTE)
        .module(module)
        .name(entry);
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

fn write_sets(context: &Context, pyramid: &DepthPyramid, depth: vk::ImageView) {
    let scene = vk::DescriptorImageInfo::default()
        .image_view(depth)
        .sampler(pyramid.sampler)
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    let sources: Vec<vk::DescriptorImageInfo> = (0..pyramid.levels.len())
        .map(|level| {
            vk::DescriptorImageInfo::default()
                .image_view(pyramid.levels[level.saturating_sub(1)])
                .image_layout(vk::ImageLayout::GENERAL)
        })
        .collect();
    let destinations: Vec<vk::DescriptorImageInfo> = pyramid
        .levels
        .iter()
        .map(|view| {
            vk::DescriptorImageInfo::default()
                .image_view(*view)
                .image_layout(vk::ImageLayout::GENERAL)
        })
        .collect();

    let mut writes = Vec::with_capacity(pyramid.sets.len() * 3);
    for (level, set) in pyramid.sets.iter().enumerate() {
        writes.push(
            vk::WriteDescriptorSet::default()
                .dst_set(*set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&scene)),
        );
        writes.push(
            vk::WriteDescriptorSet::default()
                .dst_set(*set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(&sources[level])),
        );
        writes.push(
            vk::WriteDescriptorSet::default()
                .dst_set(*set)
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                .image_info(std::slice::from_ref(&destinations[level])),
        );
    }
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

pub fn view(pyramid: &DepthPyramid) -> vk::ImageView {
    pyramid.image.view
}

pub fn image(pyramid: &DepthPyramid) -> vk::Image {
    pyramid.image.image
}

pub fn sampler(pyramid: &DepthPyramid) -> vk::Sampler {
    pyramid.sampler
}

pub fn extent(pyramid: &DepthPyramid) -> vk::Extent2D {
    pyramid.extent
}

pub fn levels(pyramid: &DepthPyramid) -> u32 {
    pyramid.levels.len() as u32
}

pub fn build(pyramid: &DepthPyramid, context: &Context, command_buffer: vk::CommandBuffer) {
    let device = &context.device;
    let count = pyramid.levels.len() as u32;
    unsafe {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            pyramid.depth_pipeline,
        );
    }
    for level in 0..count {
        if level == 1 {
            unsafe {
                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::COMPUTE,
                    pyramid.reduce_pipeline,
                );
            }
        }
        if level > 0 {
            between_levels(device, command_buffer, pyramid.image.image, count);
        }
        let destination = level_extent(pyramid.extent, level);
        let source = level_extent(pyramid.extent, level.saturating_sub(1));
        let push = LevelPush {
            destination_size: [destination.width, destination.height],
            source_size: [source.width, source.height],
        };
        unsafe {
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pyramid.pipeline_layout,
                0,
                &[pyramid.sets[level as usize]],
                &[],
            );
            device.cmd_push_constants(
                command_buffer,
                pyramid.pipeline_layout,
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
        .dst_access_mask(vk::AccessFlags2::SHADER_STORAGE_READ)
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

pub fn destroy(pyramid: &mut DepthPyramid, context: &Context) {
    unsafe {
        context
            .device
            .destroy_pipeline(pyramid.depth_pipeline, None);
        context
            .device
            .destroy_pipeline(pyramid.reduce_pipeline, None);
        context
            .device
            .destroy_pipeline_layout(pyramid.pipeline_layout, None);
        context.device.destroy_descriptor_pool(pyramid.pool, None);
        context
            .device
            .destroy_descriptor_set_layout(pyramid.layout, None);
        context.device.destroy_sampler(pyramid.sampler, None);
        for view in pyramid.levels.drain(..) {
            context.device.destroy_image_view(view, None);
        }
    }
    destroy_image(context, &mut pyramid.image);
}
