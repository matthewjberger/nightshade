use crate::renderer::context::Context;
use crate::renderer::memory::{
    Image, begin_one_shot, create_buffer, destroy_buffer, destroy_image, end_one_shot,
    wait_for_sent_work, write_buffer,
};
use crate::renderer::{Transition, transition};
use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{AllocationCreateDesc, AllocationScheme};

const TABLE_SIZE: u32 = 64;
const TABLE_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
const MATRIX_TABLE: &[u8] = include_bytes!("lookup/ltc_mat_1.bin");
const TERMS_TABLE: &[u8] = include_bytes!("lookup/ltc_mat_2.bin");

pub struct AreaTables {
    pub matrix: Image,
    pub terms: Image,
    pub sampler: vk::Sampler,
}

pub fn create(context: &Context) -> Result<AreaTables, Box<dyn std::error::Error>> {
    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE);
    let sampler = unsafe { context.device.create_sampler(&sampler_info, None)? };

    Ok(AreaTables {
        matrix: upload(context, MATRIX_TABLE, "area light matrix table")?,
        terms: upload(context, TERMS_TABLE, "area light terms table")?,
        sampler,
    })
}

fn upload(
    context: &Context,
    bytes: &[u8],
    name: &str,
) -> Result<Image, Box<dyn std::error::Error>> {
    let create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(TABLE_FORMAT)
        .extent(vk::Extent3D {
            width: TABLE_SIZE,
            height: TABLE_SIZE,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);
    let image = unsafe { context.device.create_image(&create_info, None)? };
    let requirements = unsafe { context.device.get_image_memory_requirements(image) };
    let allocation = context
        .allocator()
        .lock()
        .map_err(|error| error.to_string())?
        .allocate(&AllocationCreateDesc {
            name,
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
        .format(TABLE_FORMAT)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };

    let mut staging = create_buffer(
        context,
        "area light table staging",
        bytes.len() as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )?;
    write_buffer(&staging, bytes);

    let command_buffer = begin_one_shot(context)?;
    transition(
        &context.device,
        command_buffer,
        Transition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            source_access: vk::AccessFlags2::empty(),
            destination_access: vk::AccessFlags2::TRANSFER_WRITE,
            source_stage: vk::PipelineStageFlags2::TOP_OF_PIPE,
            destination_stage: vk::PipelineStageFlags2::TRANSFER,
        },
    );
    let region = vk::BufferImageCopy::default()
        .image_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        })
        .image_extent(vk::Extent3D {
            width: TABLE_SIZE,
            height: TABLE_SIZE,
            depth: 1,
        });
    unsafe {
        context.device.cmd_copy_buffer_to_image(
            command_buffer,
            staging.buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            std::slice::from_ref(&region),
        )
    };
    transition(
        &context.device,
        command_buffer,
        Transition {
            image,
            aspect: vk::ImageAspectFlags::COLOR,
            old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            source_access: vk::AccessFlags2::TRANSFER_WRITE,
            destination_access: vk::AccessFlags2::SHADER_SAMPLED_READ,
            source_stage: vk::PipelineStageFlags2::TRANSFER,
            destination_stage: vk::PipelineStageFlags2::FRAGMENT_SHADER,
        },
    );
    end_one_shot(context, command_buffer)?;
    wait_for_sent_work(context)?;
    destroy_buffer(context, &mut staging);

    Ok(Image {
        image,
        view,
        allocation: Some(allocation),
        format: TABLE_FORMAT,
        extent: vk::Extent2D {
            width: TABLE_SIZE,
            height: TABLE_SIZE,
        },
    })
}

pub fn destroy(tables: &mut AreaTables, context: &Context) {
    unsafe { context.device.destroy_sampler(tables.sampler, None) };
    destroy_image(context, &mut tables.matrix);
    destroy_image(context, &mut tables.terms);
}
