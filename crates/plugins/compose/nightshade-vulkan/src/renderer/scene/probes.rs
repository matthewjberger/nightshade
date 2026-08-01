use crate::renderer::context::Context;
use crate::renderer::memory::{
    Buffer, Image, begin_one_shot, create_buffer, create_storage_image, destroy_buffer,
    destroy_image, end_one_shot, transition_range,
};
use ash::vk;
use gpu_allocator::MemoryLocation;

pub const MAX_COUNTS: [u32; 3] = [16, 8, 16];

pub const IRRADIANCE_TEXELS: u32 = 8;
pub const DISTANCE_TEXELS: u32 = 16;

pub const BORDER: u32 = 1;

pub const RAYS_PER_PROBE: u32 = 64;

const OFFSET_GROUP: u32 = 64;

const IRRADIANCE_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
const DISTANCE_FORMAT: vk::Format = vk::Format::R16G16_SFLOAT;

const RAY_SIZE: u64 = std::mem::size_of::<[f32; 4]>() as u64 * 2;

const OFFSET_SIZE: u64 = std::mem::size_of::<[f32; 4]>() as u64;

pub struct Probes {
    irradiance: Image,
    distance: Image,
    rays: Buffer,
    offsets: Buffer,
    sampler: vk::Sampler,
}

pub struct Bindings {
    pub irradiance_store: u32,
    pub distance_store: u32,
    pub irradiance_read: u32,
    pub distance_read: u32,
    pub rays: u32,
    pub offsets: u32,
}

pub fn max_probe_count() -> u32 {
    MAX_COUNTS[0] * MAX_COUNTS[1] * MAX_COUNTS[2]
}

fn tile_of(texels: u32) -> u32 {
    texels + BORDER * 2
}

fn atlas_extent(texels: u32) -> vk::Extent2D {
    vk::Extent2D {
        width: MAX_COUNTS[0] * MAX_COUNTS[1] * tile_of(texels),
        height: MAX_COUNTS[2] * tile_of(texels),
    }
}

pub fn create_probes(context: &Context) -> Result<Probes, Box<dyn std::error::Error>> {
    let irradiance = create_storage_image(
        context,
        atlas_extent(IRRADIANCE_TEXELS),
        IRRADIANCE_FORMAT,
        "probe irradiance",
    )?;
    let distance = create_storage_image(
        context,
        atlas_extent(DISTANCE_TEXELS),
        DISTANCE_FORMAT,
        "probe distance",
    )?;
    let rays = create_buffer(
        context,
        "probe rays",
        max_probe_count() as u64 * RAYS_PER_PROBE as u64 * RAY_SIZE,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )?;
    let offsets = create_buffer(
        context,
        "probe offsets",
        max_probe_count() as u64 * OFFSET_SIZE,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )?;

    let command_buffer = begin_one_shot(context)?;
    let range = vk::ImageSubresourceRange {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        layer_count: 1,
    };
    for image in [&irradiance, &distance] {
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
    for buffer in [&rays, &offsets] {
        unsafe {
            context
                .device
                .cmd_fill_buffer(command_buffer, buffer.buffer, 0, vk::WHOLE_SIZE, 0)
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

    Ok(Probes {
        irradiance,
        distance,
        rays,
        offsets,
        sampler,
    })
}

pub fn destroy_probes(context: &Context, probes: &mut Probes) {
    unsafe { context.device.destroy_sampler(probes.sampler, None) };
    destroy_image(context, &mut probes.irradiance);
    destroy_image(context, &mut probes.distance);
    destroy_buffer(context, &mut probes.rays);
    destroy_buffer(context, &mut probes.offsets);
}

fn sampled(probes: &Probes, image: &Image) -> vk::DescriptorImageInfo {
    vk::DescriptorImageInfo::default()
        .image_view(image.view)
        .sampler(probes.sampler)
        .image_layout(vk::ImageLayout::GENERAL)
}

fn stored(image: &Image) -> vk::DescriptorImageInfo {
    vk::DescriptorImageInfo::default()
        .image_view(image.view)
        .image_layout(vk::ImageLayout::GENERAL)
}

pub fn bind(context: &Context, probes: &Probes, sets: &[vk::DescriptorSet], bindings: Bindings) {
    let images = [
        (
            bindings.irradiance_store,
            vk::DescriptorType::STORAGE_IMAGE,
            stored(&probes.irradiance),
        ),
        (
            bindings.distance_store,
            vk::DescriptorType::STORAGE_IMAGE,
            stored(&probes.distance),
        ),
        (
            bindings.irradiance_read,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            sampled(probes, &probes.irradiance),
        ),
        (
            bindings.distance_read,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            sampled(probes, &probes.distance),
        ),
    ];
    let buffers = [
        (
            bindings.rays,
            vk::DescriptorBufferInfo::default()
                .buffer(probes.rays.buffer)
                .offset(0)
                .range(vk::WHOLE_SIZE),
        ),
        (
            bindings.offsets,
            vk::DescriptorBufferInfo::default()
                .buffer(probes.offsets.buffer)
                .offset(0)
                .range(vk::WHOLE_SIZE),
        ),
    ];
    let mut writes = Vec::with_capacity(sets.len() * (images.len() + buffers.len()));
    for set in sets {
        for (binding, kind, info) in &images {
            writes.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(*set)
                    .dst_binding(*binding)
                    .descriptor_type(*kind)
                    .image_info(std::slice::from_ref(info)),
            );
        }
        for (binding, info) in &buffers {
            writes.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(*set)
                    .dst_binding(*binding)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(std::slice::from_ref(info)),
            );
        }
    }
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

pub struct ProbeRounds {
    offset_pipeline: vk::Pipeline,
    trace_pipeline: vk::Pipeline,
    irradiance_pipeline: vk::Pipeline,
    distance_pipeline: vk::Pipeline,
}

pub fn create_rounds(
    offset_pipeline: vk::Pipeline,
    trace_pipeline: vk::Pipeline,
    irradiance_pipeline: vk::Pipeline,
    distance_pipeline: vk::Pipeline,
) -> ProbeRounds {
    ProbeRounds {
        offset_pipeline,
        trace_pipeline,
        irradiance_pipeline,
        distance_pipeline,
    }
}

pub fn destroy_rounds(context: &Context, rounds: &mut ProbeRounds) {
    unsafe {
        context
            .device
            .destroy_pipeline(rounds.offset_pipeline, None);
        context.device.destroy_pipeline(rounds.trace_pipeline, None);
        context
            .device
            .destroy_pipeline(rounds.irradiance_pipeline, None);
        context
            .device
            .destroy_pipeline(rounds.distance_pipeline, None);
    }
}

pub fn update_probes(
    rounds: &ProbeRounds,
    probes: &Probes,
    context: &Context,
    command_buffer: vk::CommandBuffer,
    layout: vk::PipelineLayout,
    sets: [vk::DescriptorSet; 4],
    probe_count: u32,
    filled: u32,
) {
    if probe_count == 0 {
        return;
    }
    let device = &context.device;
    let push = [filled, probe_count];
    unsafe {
        device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            layout,
            0,
            &sets,
            &[],
        );
        device.cmd_push_constants(
            command_buffer,
            layout,
            vk::ShaderStageFlags::COMPUTE
                | vk::ShaderStageFlags::TASK_EXT
                | vk::ShaderStageFlags::MESH_EXT
                | vk::ShaderStageFlags::VERTEX
                | vk::ShaderStageFlags::FRAGMENT,
            0,
            bytemuck::bytes_of(&push),
        );
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            rounds.offset_pipeline,
        );
        device.cmd_dispatch(command_buffer, probe_count.div_ceil(OFFSET_GROUP), 1, 1);
    }
    rays_written_then_read(context, command_buffer, probes.offsets.buffer);
    unsafe {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            rounds.trace_pipeline,
        );
        device.cmd_dispatch(command_buffer, probe_count, 1, 1);
    }
    rays_written_then_read(context, command_buffer, probes.rays.buffer);
    unsafe {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            rounds.irradiance_pipeline,
        );
        device.cmd_dispatch(command_buffer, probe_count, 1, 1);
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            rounds.distance_pipeline,
        );
        device.cmd_dispatch(command_buffer, probe_count, 1, 1);
    }
    for image in [probes.irradiance.image, probes.distance.image] {
        atlas_written_then_read(context, command_buffer, image);
    }
}

fn rays_written_then_read(
    context: &Context,
    command_buffer: vk::CommandBuffer,
    buffer: vk::Buffer,
) {
    let barrier = vk::BufferMemoryBarrier2::default()
        .src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
        .src_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE)
        .dst_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
        .dst_access_mask(vk::AccessFlags2::SHADER_STORAGE_READ)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .buffer(buffer)
        .offset(0)
        .size(vk::WHOLE_SIZE);
    let dependency =
        vk::DependencyInfo::default().buffer_memory_barriers(std::slice::from_ref(&barrier));
    unsafe {
        context
            .device
            .cmd_pipeline_barrier2(command_buffer, &dependency)
    };
}

fn atlas_written_then_read(context: &Context, command_buffer: vk::CommandBuffer, image: vk::Image) {
    let barrier = vk::ImageMemoryBarrier2::default()
        .src_stage_mask(vk::PipelineStageFlags2::COMPUTE_SHADER)
        .src_access_mask(vk::AccessFlags2::SHADER_STORAGE_WRITE)
        .dst_stage_mask(
            vk::PipelineStageFlags2::FRAGMENT_SHADER | vk::PipelineStageFlags2::COMPUTE_SHADER,
        )
        .dst_access_mask(vk::AccessFlags2::SHADER_SAMPLED_READ)
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
