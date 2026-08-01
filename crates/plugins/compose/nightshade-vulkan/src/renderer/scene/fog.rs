use crate::renderer::context::Context;
use crate::renderer::memory::{
    Image, begin_one_shot, create_volume_image, destroy_image, end_one_shot, transition_range,
};
use ash::vk;

pub const FOG_GRID: vk::Extent3D = vk::Extent3D {
    width: 160,
    height: 90,
    depth: 64,
};
const SCATTER_GROUP: [u32; 3] = [8, 8, 4];
const MARCH_GROUP: [u32; 2] = [8, 8];
const FOG_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;

pub struct Air {
    scatter: [Image; 2],
    marched: Image,
    sampler: vk::Sampler,
}

pub struct Bindings {
    pub scatter: [u32; 2],
    pub history: [u32; 2],
    pub marched: u32,
    pub volume: u32,
}

pub fn create_air(context: &Context) -> Result<Air, Box<dyn std::error::Error>> {
    let scatter = [
        create_volume_image(context, FOG_GRID, FOG_FORMAT, "fog scatter a")?,
        create_volume_image(context, FOG_GRID, FOG_FORMAT, "fog scatter b")?,
    ];
    let marched = create_volume_image(context, FOG_GRID, FOG_FORMAT, "fog marched")?;
    let command_buffer = begin_one_shot(context)?;
    let range = vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
    };
    for image in [&scatter[0], &scatter[1], &marched] {
        transition_range(
            context,
            command_buffer,
            image.image,
            vk::ImageAspectFlags::COLOR,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
            0,
            1,
        );
        unsafe {
            context.device.cmd_clear_color_image(
                command_buffer,
                image.image,
                vk::ImageLayout::GENERAL,
                &vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
                std::slice::from_ref(&range),
            )
        };
    }
    end_one_shot(context, command_buffer)?;
    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST);
    let sampler = unsafe { context.device.create_sampler(&sampler_info, None)? };
    Ok(Air {
        scatter,
        marched,
        sampler,
    })
}

pub fn destroy_air(context: &Context, air: &mut Air) {
    unsafe { context.device.destroy_sampler(air.sampler, None) };
    for image in &mut air.scatter {
        destroy_image(context, image);
    }
    destroy_image(context, &mut air.marched);
}

pub fn air_binding(air: &Air) -> vk::DescriptorImageInfo {
    sampled(air, &air.marched)
}

fn sampled(air: &Air, image: &Image) -> vk::DescriptorImageInfo {
    vk::DescriptorImageInfo::default()
        .image_view(image.view)
        .sampler(air.sampler)
        .image_layout(vk::ImageLayout::GENERAL)
}

fn stored(image: &Image) -> vk::DescriptorImageInfo {
    vk::DescriptorImageInfo::default()
        .image_view(image.view)
        .image_layout(vk::ImageLayout::GENERAL)
}

pub struct FogRounds {
    scatter_pipeline: vk::Pipeline,
    march_pipeline: vk::Pipeline,
}

pub fn create_rounds(scatter_pipeline: vk::Pipeline, march_pipeline: vk::Pipeline) -> FogRounds {
    FogRounds {
        scatter_pipeline,
        march_pipeline,
    }
}

pub fn destroy_rounds(context: &Context, rounds: &mut FogRounds) {
    unsafe {
        context
            .device
            .destroy_pipeline(rounds.scatter_pipeline, None);
        context.device.destroy_pipeline(rounds.march_pipeline, None);
    }
}

pub fn bind(context: &Context, air: &Air, sets: &[vk::DescriptorSet], bindings: Bindings) {
    let infos = [
        (
            bindings.scatter[0],
            vk::DescriptorType::STORAGE_IMAGE,
            stored(&air.scatter[0]),
        ),
        (
            bindings.scatter[1],
            vk::DescriptorType::STORAGE_IMAGE,
            stored(&air.scatter[1]),
        ),
        (
            bindings.marched,
            vk::DescriptorType::STORAGE_IMAGE,
            stored(&air.marched),
        ),
        (
            bindings.history[0],
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            sampled(air, &air.scatter[0]),
        ),
        (
            bindings.history[1],
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            sampled(air, &air.scatter[1]),
        ),
        (
            bindings.volume,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            sampled(air, &air.marched),
        ),
    ];
    let mut writes = Vec::with_capacity(sets.len() * infos.len());
    for set in sets {
        for (binding, kind, info) in &infos {
            writes.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(*set)
                    .dst_binding(*binding)
                    .descriptor_type(*kind)
                    .image_info(std::slice::from_ref(info)),
            );
        }
    }
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

pub fn light_the_air(
    rounds: &FogRounds,
    air: &Air,
    context: &Context,
    command_buffer: vk::CommandBuffer,
    layout: vk::PipelineLayout,
    sets: [vk::DescriptorSet; 4],
    parity: usize,
) {
    let device = &context.device;
    read_then_written(
        context,
        command_buffer,
        air.scatter[parity].image,
        vk::PipelineStageFlags2::COMPUTE_SHADER,
        vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_SAMPLED_READ,
    );
    read_then_written(
        context,
        command_buffer,
        air.marched.image,
        vk::PipelineStageFlags2::FRAGMENT_SHADER,
        vk::AccessFlags2::SHADER_SAMPLED_READ,
    );
    unsafe {
        device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            layout,
            0,
            &sets,
            &[],
        );
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            rounds.scatter_pipeline,
        );
        device.cmd_dispatch(
            command_buffer,
            FOG_GRID.width.div_ceil(SCATTER_GROUP[0]),
            FOG_GRID.height.div_ceil(SCATTER_GROUP[1]),
            FOG_GRID.depth.div_ceil(SCATTER_GROUP[2]),
        );
    }
    written_then_read(
        context,
        command_buffer,
        air.scatter[parity].image,
        vk::PipelineStageFlags2::COMPUTE_SHADER,
        vk::AccessFlags2::SHADER_STORAGE_READ | vk::AccessFlags2::SHADER_SAMPLED_READ,
    );
    unsafe {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            rounds.march_pipeline,
        );
        device.cmd_dispatch(
            command_buffer,
            FOG_GRID.width.div_ceil(MARCH_GROUP[0]),
            FOG_GRID.height.div_ceil(MARCH_GROUP[1]),
            1,
        );
    }
    written_then_read(
        context,
        command_buffer,
        air.marched.image,
        vk::PipelineStageFlags2::FRAGMENT_SHADER,
        vk::AccessFlags2::SHADER_SAMPLED_READ,
    );
}

fn written_then_read(
    context: &Context,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    read_stage: vk::PipelineStageFlags2,
    read_access: vk::AccessFlags2,
) {
    grid_barrier(
        context,
        command_buffer,
        image,
        vk::PipelineStageFlags2::COMPUTE_SHADER,
        vk::AccessFlags2::SHADER_STORAGE_WRITE,
        read_stage,
        read_access,
    );
}

fn read_then_written(
    context: &Context,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    read_stage: vk::PipelineStageFlags2,
    read_access: vk::AccessFlags2,
) {
    grid_barrier(
        context,
        command_buffer,
        image,
        read_stage,
        read_access,
        vk::PipelineStageFlags2::COMPUTE_SHADER,
        vk::AccessFlags2::SHADER_STORAGE_WRITE,
    );
}

fn grid_barrier(
    context: &Context,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    src_stage: vk::PipelineStageFlags2,
    src_access: vk::AccessFlags2,
    dst_stage: vk::PipelineStageFlags2,
    dst_access: vk::AccessFlags2,
) {
    let barrier = vk::ImageMemoryBarrier2::default()
        .src_stage_mask(src_stage)
        .src_access_mask(src_access)
        .dst_stage_mask(dst_stage)
        .dst_access_mask(dst_access)
        .old_layout(vk::ImageLayout::GENERAL)
        .new_layout(vk::ImageLayout::GENERAL)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let dependency =
        vk::DependencyInfo::default().image_memory_barriers(std::slice::from_ref(&barrier));
    unsafe {
        context
            .device
            .cmd_pipeline_barrier2(command_buffer, &dependency)
    };
}
