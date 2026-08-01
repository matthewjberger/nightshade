use crate::renderer::context::{Context, Lane, SentOnce};
use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub allocation: Option<Allocation>,
    pub size: u64,
}

pub fn create_buffer(
    context: &Context,
    name: &str,
    size: u64,
    usage: vk::BufferUsageFlags,
    location: MemoryLocation,
) -> Result<Buffer, Box<dyn std::error::Error>> {
    let families = context.families_in_use();
    create_buffer_across(context, name, size, usage, location, &families)
}

fn create_buffer_across(
    context: &Context,
    name: &str,
    size: u64,
    usage: vk::BufferUsageFlags,
    location: MemoryLocation,
    families: &[u32],
) -> Result<Buffer, Box<dyn std::error::Error>> {
    let mut create_info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    if families.len() > 1 {
        create_info = create_info
            .sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(families);
    }
    let buffer = unsafe { context.device.create_buffer(&create_info, None)? };
    let requirements = unsafe { context.device.get_buffer_memory_requirements(buffer) };
    let allocation = context
        .allocator()
        .lock()
        .map_err(|error| error.to_string())?
        .allocate(&AllocationCreateDesc {
            name,
            requirements,
            location,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        })?;
    unsafe {
        context
            .device
            .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())?;
    }
    Ok(Buffer {
        buffer,
        allocation: Some(allocation),
        size,
    })
}

pub fn write_buffer<T: bytemuck::Pod>(buffer: &Buffer, data: &[T]) {
    write_buffer_at(buffer, 0, data);
}

pub fn write_buffer_at<T: bytemuck::Pod>(buffer: &Buffer, offset: u64, data: &[T]) {
    let Some(allocation) = buffer.allocation.as_ref() else {
        return;
    };
    let Some(mapped) = allocation.mapped_ptr() else {
        return;
    };
    if offset >= buffer.size {
        return;
    }
    let bytes = bytemuck::cast_slice::<T, u8>(data);
    let length = bytes.len().min((buffer.size - offset) as usize);
    unsafe {
        std::ptr::copy_nonoverlapping(
            bytes.as_ptr(),
            (mapped.as_ptr() as *mut u8).add(offset as usize),
            length,
        );
    }
}

pub fn read_buffer<T: bytemuck::Pod>(buffer: &Buffer, offset: u64, count: usize) -> Vec<T> {
    let Some(allocation) = buffer.allocation.as_ref() else {
        return Vec::new();
    };
    let Some(mapped) = allocation.mapped_ptr() else {
        return Vec::new();
    };
    let bytes = count * std::mem::size_of::<T>();
    if offset + bytes as u64 > buffer.size {
        return Vec::new();
    }
    let mut values = vec![T::zeroed(); count];
    unsafe {
        std::ptr::copy_nonoverlapping(
            (mapped.as_ptr() as *const u8).add(offset as usize),
            bytemuck::cast_slice_mut(&mut values).as_mut_ptr(),
            bytes,
        );
    }
    values
}

pub fn destroy_buffer(context: &Context, buffer: &mut Buffer) {
    unsafe { context.device.destroy_buffer(buffer.buffer, None) };
    let Some(allocation) = buffer.allocation.take() else {
        return;
    };
    let Ok(mut allocator) = context.allocator().lock() else {
        return;
    };
    if let Err(error) = allocator.free(allocation) {
        tracing::error!("a buffer allocation was not released: {error}");
    }
}

pub struct Image {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub allocation: Option<Allocation>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
}

pub fn create_depth_image(
    context: &Context,
    extent: vk::Extent2D,
) -> Result<Image, Box<dyn std::error::Error>> {
    let format = vk::Format::D32_SFLOAT;
    let create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width: extent.width.max(1),
            height: extent.height.max(1),
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
                | vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_SRC,
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
            name: "depth target",
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
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::DEPTH,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };

    Ok(Image {
        image,
        view,
        allocation: Some(allocation),
        format,
        extent,
    })
}

pub fn create_color_image(
    context: &Context,
    extent: vk::Extent2D,
    format: vk::Format,
    name: &str,
) -> Result<Image, Box<dyn std::error::Error>> {
    let create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width: extent.width.max(1),
            height: extent.height.max(1),
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(
            vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_SRC,
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
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };

    Ok(Image {
        image,
        view,
        allocation: Some(allocation),
        format,
        extent,
    })
}

pub fn create_pyramid_image(
    context: &Context,
    extent: vk::Extent2D,
    levels: u32,
    name: &str,
) -> Result<Image, Box<dyn std::error::Error>> {
    create_chain_image(context, extent, levels, vk::Format::R32_SFLOAT, name)
}

pub fn create_chain_image(
    context: &Context,
    extent: vk::Extent2D,
    levels: u32,
    format: vk::Format,
    name: &str,
) -> Result<Image, Box<dyn std::error::Error>> {
    let create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width: extent.width.max(1),
            height: extent.height.max(1),
            depth: 1,
        })
        .mip_levels(levels.max(1))
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::SAMPLED)
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
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: levels.max(1),
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };

    Ok(Image {
        image,
        view,
        allocation: Some(allocation),
        format,
        extent,
    })
}

pub fn create_storage_image(
    context: &Context,
    extent: vk::Extent2D,
    format: vk::Format,
    name: &str,
) -> Result<Image, Box<dyn std::error::Error>> {
    let create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width: extent.width.max(1),
            height: extent.height.max(1),
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(
            vk::ImageUsageFlags::STORAGE
                | vk::ImageUsageFlags::SAMPLED
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
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };
    Ok(Image {
        image,
        view,
        allocation: Some(allocation),
        format,
        extent,
    })
}

pub fn create_volume_image(
    context: &Context,
    extent: vk::Extent3D,
    format: vk::Format,
    name: &str,
) -> Result<Image, Box<dyn std::error::Error>> {
    let create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_3D)
        .format(format)
        .extent(vk::Extent3D {
            width: extent.width.max(1),
            height: extent.height.max(1),
            depth: extent.depth.max(1),
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(
            vk::ImageUsageFlags::STORAGE
                | vk::ImageUsageFlags::SAMPLED
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
        .view_type(vk::ImageViewType::TYPE_3D)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };

    Ok(Image {
        image,
        view,
        allocation: Some(allocation),
        format,
        extent: vk::Extent2D {
            width: extent.width,
            height: extent.height,
        },
    })
}

pub fn destroy_image(context: &Context, image: &mut Image) {
    unsafe {
        context.device.destroy_image_view(image.view, None);
        context.device.destroy_image(image.image, None);
    }
    let Some(allocation) = image.allocation.take() else {
        return;
    };
    let Ok(mut allocator) = context.allocator().lock() else {
        return;
    };
    if let Err(error) = allocator.free(allocation) {
        tracing::error!("an image allocation was not released: {error}");
    }
}

pub fn upload_to_buffer(
    context: &Context,
    target: &Buffer,
    offset: u64,
    bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    if bytes.is_empty() || offset >= target.size {
        return Ok(());
    }
    let length = (bytes.len() as u64).min(target.size - offset);
    let mut staging = create_buffer(
        context,
        "staging upload",
        length,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )?;
    if let Some(allocation) = staging.allocation.as_ref()
        && let Some(mapped) = allocation.mapped_ptr()
    {
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                mapped.as_ptr() as *mut u8,
                length as usize,
            );
        }
    }

    let lane = context.upload_lane();
    let command_buffer = begin_one_shot_on(context, lane)?;
    let region = vk::BufferCopy::default().dst_offset(offset).size(length);
    unsafe {
        context.device.cmd_copy_buffer(
            command_buffer,
            staging.buffer,
            target.buffer,
            std::slice::from_ref(&region),
        );
    }
    end_one_shot_on(context, lane, command_buffer)?;
    retire_buffer_on(context, lane, &mut staging);
    Ok(())
}

pub fn begin_one_shot(context: &Context) -> Result<vk::CommandBuffer, Box<dyn std::error::Error>> {
    begin_one_shot_on(context, &context.graphics)
}

pub fn begin_one_shot_on(
    context: &Context,
    lane: &Lane,
) -> Result<vk::CommandBuffer, Box<dyn std::error::Error>> {
    reclaim_sent_work(context);
    let allocate_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(lane.pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let buffers = unsafe { context.device.allocate_command_buffers(&allocate_info)? };
    let begin_info =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe {
        context
            .device
            .begin_command_buffer(buffers[0], &begin_info)?
    };
    Ok(buffers[0])
}

pub fn end_one_shot(
    context: &Context,
    command_buffer: vk::CommandBuffer,
) -> Result<(), Box<dyn std::error::Error>> {
    end_one_shot_on(context, &context.graphics, command_buffer)
}

pub fn end_one_shot_on(
    context: &Context,
    lane: &Lane,
    command_buffer: vk::CommandBuffer,
) -> Result<(), Box<dyn std::error::Error>> {
    submit_one_shot(context, lane, command_buffer, &[])
}

pub fn end_one_shot_after(
    context: &Context,
    lane: &Lane,
    command_buffer: vk::CommandBuffer,
    written_by: &Lane,
    reached: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let timeline = {
        let Ok(work) = written_by.work.lock() else {
            return Err("the sent work was not reachable".into());
        };
        work.timeline
    };
    let wait = vk::SemaphoreSubmitInfo::default()
        .semaphore(timeline)
        .value(reached)
        .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS);
    submit_one_shot(context, lane, command_buffer, std::slice::from_ref(&wait))
}

fn submit_one_shot(
    context: &Context,
    lane: &Lane,
    command_buffer: vk::CommandBuffer,
    waits: &[vk::SemaphoreSubmitInfo],
) -> Result<(), Box<dyn std::error::Error>> {
    let submit_buffer = vk::CommandBufferSubmitInfo::default().command_buffer(command_buffer);
    let Ok(mut work) = lane.work.lock() else {
        return Err("the sent work was not reachable".into());
    };
    let done_at = work.sent + 1;
    let signal = vk::SemaphoreSubmitInfo::default()
        .semaphore(work.timeline)
        .value(done_at)
        .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS);
    let submit = vk::SubmitInfo2::default()
        .wait_semaphore_infos(waits)
        .command_buffer_infos(std::slice::from_ref(&submit_buffer))
        .signal_semaphore_infos(std::slice::from_ref(&signal));
    unsafe {
        context.device.end_command_buffer(command_buffer)?;
        context
            .device
            .queue_submit2(lane.queue, std::slice::from_ref(&submit), vk::Fence::null())
            .inspect_err(|outcome| context.note(*outcome))?;
    }
    work.sent = done_at;
    work.waiting.push(SentOnce {
        done_at,
        command_buffer,
        buffers: Vec::new(),
        allocations: Vec::new(),
    });
    Ok(())
}

pub fn wait_for_count(
    context: &Context,
    lane: &Lane,
    reached: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    if reached == 0 {
        return Ok(());
    }
    let timeline = {
        let Ok(work) = lane.work.lock() else {
            return Err("the sent work was not reachable".into());
        };
        work.timeline
    };
    let wait = vk::SemaphoreWaitInfo::default()
        .semaphores(std::slice::from_ref(&timeline))
        .values(std::slice::from_ref(&reached));
    unsafe { context.device.wait_semaphores(&wait, u64::MAX)? };
    reclaim_sent_work(context);
    Ok(())
}

pub fn sent_count(lane: &Lane) -> u64 {
    lane.work.lock().map(|work| work.sent).unwrap_or(0)
}

pub fn wait_for_sent_work(context: &Context) -> Result<(), Box<dyn std::error::Error>> {
    for lane in context.lanes() {
        let (timeline, sent) = {
            let Ok(work) = lane.work.lock() else {
                return Err("the sent work was not reachable".into());
            };
            (work.timeline, work.sent)
        };
        if sent > 0 {
            let wait = vk::SemaphoreWaitInfo::default()
                .semaphores(std::slice::from_ref(&timeline))
                .values(std::slice::from_ref(&sent));
            unsafe { context.device.wait_semaphores(&wait, u64::MAX)? };
        }
    }
    reclaim_sent_work(context);
    Ok(())
}

pub fn retire_buffer(context: &Context, buffer: &mut Buffer) {
    retire_buffer_on(context, &context.graphics, buffer);
}

pub fn retire_buffer_on(context: &Context, lane: &Lane, buffer: &mut Buffer) {
    let Ok(mut work) = lane.work.lock() else {
        return;
    };
    let Some(entry) = work.waiting.last_mut() else {
        drop(work);
        destroy_buffer(context, buffer);
        return;
    };
    entry.buffers.push(buffer.buffer);
    if let Some(allocation) = buffer.allocation.take() {
        entry.allocations.push(allocation);
    }
}

pub fn reclaim_sent_work(context: &Context) {
    for lane in context.lanes() {
        reclaim_lane(context, lane);
    }
}

fn reclaim_lane(context: &Context, lane: &Lane) {
    let Ok(mut work) = lane.work.lock() else {
        return;
    };
    let reached = unsafe { context.device.get_semaphore_counter_value(work.timeline) };
    let Ok(reached) = reached else {
        return;
    };
    let mut done = Vec::new();
    let mut index = 0;
    while index < work.waiting.len() {
        if work.waiting[index].done_at <= reached {
            done.push(work.waiting.remove(index));
        } else {
            index += 1;
        }
    }
    drop(work);

    let Ok(mut allocator) = context.allocator().lock() else {
        return;
    };
    for entry in done {
        unsafe {
            context
                .device
                .free_command_buffers(lane.pool, std::slice::from_ref(&entry.command_buffer));
            for buffer in entry.buffers {
                context.device.destroy_buffer(buffer, None);
            }
        }
        for allocation in entry.allocations {
            if let Err(error) = allocator.free(allocation) {
                tracing::error!("the memory was not given back: {error}");
            }
        }
    }
}

pub fn transition(
    context: &Context,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) {
    transition_range(
        context,
        command_buffer,
        image,
        vk::ImageAspectFlags::COLOR,
        old_layout,
        new_layout,
        0,
        1,
    );
}

pub fn transition_range(
    context: &Context,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    aspect_mask: vk::ImageAspectFlags,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    base_mip_level: u32,
    level_count: u32,
) {
    let barrier = vk::ImageMemoryBarrier2::default()
        .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
        .src_access_mask(vk::AccessFlags2::MEMORY_WRITE)
        .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
        .dst_access_mask(vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE)
        .old_layout(old_layout)
        .new_layout(new_layout)
        .image(image)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level,
            level_count,
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

#[derive(Clone, Copy)]
pub enum Handover {
    Release,
    Acquire,
}

pub fn hand_over_image(
    context: &Context,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    layout: vk::ImageLayout,
    level_count: u32,
    from: &Lane,
    to: &Lane,
    half: Handover,
) {
    let barrier = match half {
        Handover::Release => vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .src_access_mask(vk::AccessFlags2::MEMORY_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::NONE)
            .dst_access_mask(vk::AccessFlags2::NONE),
        Handover::Acquire => vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::NONE)
            .src_access_mask(vk::AccessFlags2::NONE)
            .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .dst_access_mask(vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE),
    };
    let barrier = barrier
        .old_layout(layout)
        .new_layout(layout)
        .src_queue_family_index(from.family)
        .dst_queue_family_index(to.family)
        .image(image)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count,
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

pub fn release(context: &Context, allocation: Option<Allocation>) {
    let Some(allocation) = allocation else {
        return;
    };
    let Ok(mut allocator) = context.allocator().lock() else {
        return;
    };
    if let Err(error) = allocator.free(allocation) {
        tracing::error!("a glyph atlas allocation was not released: {error}");
    }
}
