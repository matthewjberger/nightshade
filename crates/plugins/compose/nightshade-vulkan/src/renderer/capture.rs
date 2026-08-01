use crate::renderer::context::Context;
use crate::renderer::memory::{
    begin_one_shot, create_buffer, destroy_buffer, end_one_shot, transition, wait_for_sent_work,
};
use ash::vk;
use gpu_allocator::MemoryLocation;
use std::path::PathBuf;

pub struct CaptureRequest {
    pub path: Option<PathBuf>,
    pub max_dimension: Option<u32>,
}

pub fn capture_image(
    context: &Context,
    image: vk::Image,
    format: vk::Format,
    extent: vk::Extent2D,
    paper_white: f32,
    request: &CaptureRequest,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let width = extent.width;
    let height = extent.height;
    let size = width as u64 * height as u64 * bytes_per_pixel(format);
    let mut staging = create_buffer(
        context,
        "screenshot staging",
        size,
        vk::BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuToCpu,
    )?;

    let command_buffer = begin_one_shot(context)?;
    transition(
        context,
        command_buffer,
        image,
        vk::ImageLayout::PRESENT_SRC_KHR,
        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
    );
    let region = vk::BufferImageCopy::default()
        .image_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        })
        .image_extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        });
    unsafe {
        context.device.cmd_copy_image_to_buffer(
            command_buffer,
            image,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            staging.buffer,
            std::slice::from_ref(&region),
        )
    };
    transition(
        context,
        command_buffer,
        image,
        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        vk::ImageLayout::PRESENT_SRC_KHR,
    );
    end_one_shot(context, command_buffer)?;
    wait_for_sent_work(context)?;

    let pixels = read_back(&staging, size as usize, format, paper_white);
    destroy_buffer(context, &mut staging);

    let Some(picture) = image::RgbaImage::from_raw(width, height, pixels) else {
        return Err("the screenshot pixels did not fill the image".into());
    };
    let picture = match request.max_dimension {
        Some(limit) if width.max(height) > limit => {
            let scale = limit as f32 / width.max(height) as f32;
            image::imageops::resize(
                &picture,
                ((width as f32 * scale) as u32).max(1),
                ((height as f32 * scale) as u32).max(1),
                image::imageops::FilterType::Lanczos3,
            )
        }
        _ => picture,
    };

    let path = request.path.clone().unwrap_or_else(default_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    picture.save(&path)?;
    Ok(path)
}

fn bytes_per_pixel(format: vk::Format) -> u64 {
    match format {
        vk::Format::R16G16B16A16_SFLOAT => 8,
        _ => 4,
    }
}

fn read_back(
    staging: &crate::renderer::memory::Buffer,
    size: usize,
    format: vk::Format,
    paper_white: f32,
) -> Vec<u8> {
    let mut bytes = vec![0u8; size];
    let Some(allocation) = staging.allocation.as_ref() else {
        return bytes;
    };
    let Some(mapped) = allocation.mapped_ptr() else {
        return bytes;
    };
    unsafe {
        std::ptr::copy_nonoverlapping(mapped.as_ptr() as *const u8, bytes.as_mut_ptr(), size);
    }
    match format {
        vk::Format::R16G16B16A16_SFLOAT => bytes
            .as_chunks::<8>()
            .0
            .iter()
            .flat_map(|pixel| {
                let channel = |at: usize| {
                    let nits = half::f16::from_le_bytes([pixel[at], pixel[at + 1]]).to_f32() * 80.0;
                    paper_byte(nits, paper_white)
                };
                [channel(0), channel(2), channel(4), 255]
            })
            .collect(),
        vk::Format::A2B10G10R10_UNORM_PACK32 => bytes
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|pixel| {
                let packed = u32::from_le_bytes(*pixel);
                let channel = |shift: u32| {
                    let coded = ((packed >> shift) & 0x3FF) as f32 / 1023.0;
                    paper_byte(pq_nits(coded), paper_white)
                };
                [channel(0), channel(10), channel(20), 255]
            })
            .collect(),
        _ => {
            if matches!(
                format,
                vk::Format::B8G8R8A8_UNORM | vk::Format::B8G8R8A8_SRGB
            ) {
                for pixel in bytes.as_chunks_mut::<4>().0 {
                    pixel.swap(0, 2);
                }
            }
            for pixel in bytes.as_chunks_mut::<4>().0 {
                pixel[3] = 255;
            }
            bytes
        }
    }
}

fn paper_byte(nits: f32, paper_white: f32) -> u8 {
    let linear = (nits / paper_white.max(1.0)).clamp(0.0, 1.0);
    let encoded = if linear <= 0.0031308 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round() as u8
}

fn pq_nits(coded: f32) -> f32 {
    const M1: f32 = 0.159_301_76;
    const M2: f32 = 78.84375;
    const C1: f32 = 0.8359375;
    const C2: f32 = 18.851_563;
    const C3: f32 = 18.6875;
    let unpowered = coded.max(0.0).powf(1.0 / M2);
    let normalized = ((unpowered - C1).max(0.0) / (C2 - C3 * unpowered)).powf(1.0 / M1);
    normalized * 10000.0
}

fn default_path() -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or(0);
    PathBuf::from(format!("screenshot_{stamp}.png"))
}
