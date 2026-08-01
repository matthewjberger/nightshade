use crate::renderer::context::Context;
use crate::renderer::memory::{Buffer, create_buffer, destroy_buffer, read_buffer};
use ash::vk;
use gpu_allocator::MemoryLocation;

pub const SAMPLE_SIZE: u32 = 5;

const TEXELS: usize = (SAMPLE_SIZE * SAMPLE_SIZE) as usize;
const DEPTH_BYTES: u64 = (TEXELS * std::mem::size_of::<f32>()) as u64;
const BUFFER_BYTES: u64 = DEPTH_BYTES * 2;

#[derive(Clone, Copy)]
pub struct Window {
    pub center: (u32, u32),
    pub extent: (u32, u32),
}

pub struct Samples {
    pub window: Window,
    pub depth: Vec<f32>,
    pub entity_ids: Vec<u32>,
}

pub struct Picking {
    buffers: Vec<Buffer>,
    on_the_way: Vec<Option<Window>>,
    pub wanted: Option<(u32, u32)>,
    pub arrived: Option<Samples>,
}

pub fn create(
    context: &Context,
    frames_in_flight: usize,
) -> Result<Picking, Box<dyn std::error::Error>> {
    let mut buffers = Vec::with_capacity(frames_in_flight);
    for _ in 0..frames_in_flight {
        buffers.push(create_buffer(
            context,
            "pick readback",
            BUFFER_BYTES,
            vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::GpuToCpu,
        )?);
    }
    Ok(Picking {
        buffers,
        on_the_way: vec![None; frames_in_flight],
        wanted: None,
        arrived: None,
    })
}

pub fn destroy(picking: &mut Picking, context: &Context) {
    for buffer in &mut picking.buffers {
        destroy_buffer(context, buffer);
    }
    picking.buffers.clear();
}

pub fn collect(picking: &mut Picking, frame_index: usize) {
    let Some(window) = picking
        .on_the_way
        .get_mut(frame_index)
        .and_then(Option::take)
    else {
        return;
    };
    let Some(buffer) = picking.buffers.get(frame_index) else {
        return;
    };
    let depth: Vec<f32> = read_buffer(buffer, 0, TEXELS);
    let entity_ids: Vec<u32> = read_buffer(buffer, DEPTH_BYTES, TEXELS);
    picking.arrived = Some(Samples {
        window,
        depth,
        entity_ids,
    });
}

pub fn copy(
    picking: &mut Picking,
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    frame_index: usize,
    depth: vk::Image,
    entity_id: vk::Image,
    extent: vk::Extent2D,
    point: (u32, u32),
) {
    let Some(buffer) = picking.buffers.get(frame_index) else {
        return;
    };
    if extent.width < SAMPLE_SIZE || extent.height < SAMPLE_SIZE {
        return;
    }
    let half = SAMPLE_SIZE / 2;
    let center = (
        point.0.clamp(half, extent.width - 1 - half),
        point.1.clamp(half, extent.height - 1 - half),
    );
    let offset = vk::Offset3D {
        x: (center.0 - half) as i32,
        y: (center.1 - half) as i32,
        z: 0,
    };
    let window = vk::Extent3D {
        width: SAMPLE_SIZE,
        height: SAMPLE_SIZE,
        depth: 1,
    };
    let region = |aspect, buffer_offset| {
        vk::BufferImageCopy::default()
            .buffer_offset(buffer_offset)
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: aspect,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image_offset(offset)
            .image_extent(window)
    };
    unsafe {
        device.cmd_copy_image_to_buffer(
            command_buffer,
            depth,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            buffer.buffer,
            std::slice::from_ref(&region(vk::ImageAspectFlags::DEPTH, 0)),
        );
        device.cmd_copy_image_to_buffer(
            command_buffer,
            entity_id,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            buffer.buffer,
            std::slice::from_ref(&region(vk::ImageAspectFlags::COLOR, DEPTH_BYTES)),
        );
    }
    if let Some(slot) = picking.on_the_way.get_mut(frame_index) {
        *slot = Some(Window {
            center,
            extent: (extent.width, extent.height),
        });
    }
}
