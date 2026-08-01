use crate::renderer::context::{Context, FRAMES_IN_FLIGHT};
use crate::renderer::memory::release;
use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};

pub const ATLAS_SIZE: u32 = 8192;
pub const CASCADE_COUNT: usize = 4;
pub const SLOT_SIZE: u32 = ATLAS_SIZE / 2;

pub const DEPTH_FORMAT: vk::Format = vk::Format::D32_SFLOAT;

pub const LOCAL_ATLAS_SIZE: u32 = 4096;

pub const LOCAL_SLOT_WIDEST: u32 = LOCAL_ATLAS_SIZE / 4;
pub const LOCAL_SLOT_NARROWEST: u32 = LOCAL_SLOT_WIDEST / 8;

pub struct ShadowAtlas {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub local_image: vk::Image,
    pub local_view: vk::ImageView,
    local_allocation: Option<Allocation>,
    allocation: Option<Allocation>,
    sampler: vk::Sampler,
    plain_sampler: vk::Sampler,
    pub layout: vk::DescriptorSetLayout,
    pub sets: [vk::DescriptorSet; FRAMES_IN_FLIGHT],
    waiting: Option<vk::AccelerationStructureKHR>,
    stale: [bool; FRAMES_IN_FLIGHT],
    pool: vk::DescriptorPool,
}

fn create_atlas_image(
    context: &Context,
    name: &'static str,
    size: u32,
) -> Result<(vk::Image, vk::ImageView, Allocation), Box<dyn std::error::Error>> {
    let create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(DEPTH_FORMAT)
        .extent(vk::Extent3D {
            width: size,
            height: size,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
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
        .format(DEPTH_FORMAT)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::DEPTH,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };
    Ok((image, view, allocation))
}

pub fn create_atlas(context: &Context) -> Result<ShadowAtlas, Box<dyn std::error::Error>> {
    let (local_image, local_view, local_allocation) =
        create_atlas_image(context, "local shadow atlas", LOCAL_ATLAS_SIZE)?;
    let (image, view, allocation) = create_atlas_image(context, "shadow atlas", ATLAS_SIZE)?;

    let sampler = create_sampler(context)?;
    let plain_sampler = create_plain_sampler(context)?;
    let layout = create_layout(context)?;
    let pool = create_pool(context)?;
    let layouts = [layout; FRAMES_IN_FLIGHT];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let allocated = unsafe { context.device.allocate_descriptor_sets(&allocate_info)? };
    let mut sets = [vk::DescriptorSet::null(); FRAMES_IN_FLIGHT];
    sets.copy_from_slice(&allocated);

    let atlas = ShadowAtlas {
        image,
        view,
        local_image,
        local_view,
        local_allocation: Some(local_allocation),
        allocation: Some(allocation),
        sampler,
        plain_sampler,
        layout,
        sets,
        waiting: None,
        stale: [false; FRAMES_IN_FLIGHT],
        pool,
    };
    write_descriptor(context, &atlas);
    Ok(atlas)
}

pub fn destroy_atlas(context: &Context, atlas: &mut ShadowAtlas) {
    unsafe {
        context.device.destroy_descriptor_pool(atlas.pool, None);
        context
            .device
            .destroy_descriptor_set_layout(atlas.layout, None);
        context.device.destroy_sampler(atlas.sampler, None);
        context.device.destroy_sampler(atlas.plain_sampler, None);
        context.device.destroy_image_view(atlas.view, None);
        context.device.destroy_image(atlas.image, None);
        context.device.destroy_image_view(atlas.local_view, None);
        context.device.destroy_image(atlas.local_image, None);
    }
    release(context, atlas.allocation.take());
    release(context, atlas.local_allocation.take());
}

pub fn slot_origin(cascade: usize) -> (u32, u32) {
    (
        (cascade as u32 % 2) * SLOT_SIZE,
        (cascade as u32 / 2) * SLOT_SIZE,
    )
}

fn write_descriptor(context: &Context, atlas: &ShadowAtlas) {
    let info = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(atlas.view)
        .sampler(atlas.sampler);
    let local = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(atlas.local_view)
        .sampler(atlas.sampler);
    let depths = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(atlas.view)
        .sampler(atlas.plain_sampler);
    let local_depths = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(atlas.local_view)
        .sampler(atlas.plain_sampler);
    let mut writes = Vec::with_capacity(4 * FRAMES_IN_FLIGHT);
    for set in atlas.sets {
        writes.extend([
            vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&info)),
            vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&local)),
            vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(2)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&depths)),
            vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(3)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&local_depths)),
        ]);
    }
    unsafe { context.device.update_descriptor_sets(&writes, &[]) };
}

fn create_sampler(context: &Context) -> Result<vk::Sampler, Box<dyn std::error::Error>> {
    let create_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
        .compare_enable(true)
        .compare_op(vk::CompareOp::GREATER_OR_EQUAL);
    Ok(unsafe { context.device.create_sampler(&create_info, None)? })
}

fn create_plain_sampler(context: &Context) -> Result<vk::Sampler, Box<dyn std::error::Error>> {
    let create_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::NEAREST)
        .min_filter(vk::Filter::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST);
    Ok(unsafe { context.device.create_sampler(&create_info, None)? })
}

fn create_layout(context: &Context) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let stages = vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE;
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(stages),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(stages),
        vk::DescriptorSetLayoutBinding::default()
            .binding(2)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(stages),
        vk::DescriptorSetLayoutBinding::default()
            .binding(3)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(stages),
        vk::DescriptorSetLayoutBinding::default()
            .binding(4)
            .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
            .descriptor_count(1)
            .stage_flags(stages),
    ];
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

fn create_pool(context: &Context) -> Result<vk::DescriptorPool, Box<dyn std::error::Error>> {
    let held = FRAMES_IN_FLIGHT as u32;
    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(4 * held),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
            .descriptor_count(held),
    ];
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&sizes)
        .max_sets(held);
    Ok(unsafe { context.device.create_descriptor_pool(&create_info, None)? })
}

pub fn point_at_world(shadows: &mut ShadowAtlas, world: vk::AccelerationStructureKHR) {
    shadows.waiting = Some(world);
    shadows.stale = [true; FRAMES_IN_FLIGHT];
}

pub fn settle_world_set(context: &Context, shadows: &mut ShadowAtlas, index: usize) {
    if index >= FRAMES_IN_FLIGHT || !shadows.stale[index] {
        return;
    }
    let Some(world) = shadows.waiting else {
        return;
    };
    write_world(context, shadows.sets[index], world);
    shadows.stale[index] = false;
}

fn write_world(context: &Context, set: vk::DescriptorSet, world: vk::AccelerationStructureKHR) {
    let mut holds = vk::WriteDescriptorSetAccelerationStructureKHR::default()
        .acceleration_structures(std::slice::from_ref(&world));
    let mut write = vk::WriteDescriptorSet::default()
        .dst_set(set)
        .dst_binding(4)
        .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
        .push_next(&mut holds);
    write.descriptor_count = 1;
    unsafe { context.device.update_descriptor_sets(&[write], &[]) };
}
