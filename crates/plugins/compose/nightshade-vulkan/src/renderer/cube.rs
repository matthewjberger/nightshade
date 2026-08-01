use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{AllocationCreateDesc, AllocationScheme};

use crate::renderer::context::Context;
use crate::renderer::memory::release;

pub struct CubeImage {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub level_views: Vec<vk::ImageView>,
    pub size: u32,
    pub levels: u32,
    allocation: Option<gpu_allocator::vulkan::Allocation>,
}

pub const CUBE_FORMAT: vk::Format = vk::Format::R16G16B16A16_SFLOAT;

pub fn create_cube_image(
    context: &Context,
    size: u32,
    levels: u32,
    name: &str,
) -> Result<CubeImage, Box<dyn std::error::Error>> {
    let create_info = vk::ImageCreateInfo::default()
        .flags(vk::ImageCreateFlags::CUBE_COMPATIBLE)
        .image_type(vk::ImageType::TYPE_2D)
        .format(CUBE_FORMAT)
        .extent(vk::Extent3D {
            width: size,
            height: size,
            depth: 1,
        })
        .mip_levels(levels)
        .array_layers(6)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(
            vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::STORAGE
                | vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST,
        )
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
        .view_type(vk::ImageViewType::CUBE)
        .format(CUBE_FORMAT)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: levels,
            base_array_layer: 0,
            layer_count: 6,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };

    let mut level_views = Vec::with_capacity(levels as usize);
    for level in 0..levels {
        let level_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D_ARRAY)
            .format(CUBE_FORMAT)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: level,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 6,
            });
        level_views.push(unsafe { context.device.create_image_view(&level_info, None)? });
    }

    Ok(CubeImage {
        image,
        view,
        level_views,
        size,
        levels,
        allocation: Some(allocation),
    })
}

pub fn destroy_cube_image(context: &Context, cube: &mut CubeImage) {
    unsafe {
        for view in cube.level_views.drain(..) {
            context.device.destroy_image_view(view, None);
        }
        context.device.destroy_image_view(cube.view, None);
        context.device.destroy_image(cube.image, None);
    }
    release(context, cube.allocation.take());
}

pub fn level_count(size: u32) -> u32 {
    32 - size.leading_zeros()
}
