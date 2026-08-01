use crate::renderer::context::Context;
use crate::renderer::memory::{
    begin_one_shot, create_buffer, end_one_shot, release, retire_buffer, transition,
    wait_for_sent_work, write_buffer,
};
use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};
use nightshade_text::contract::data::cosmic_text::{CacheKey, FontSystem, SwashCache};
use nightshade_text::contract::data::glyph_packer::{
    ATLAS_INITIAL_SIZE, GlyphAlloc, GlyphPacker, GlyphWork, RasterizedGlyph, ensure_glyph_slot,
};

pub struct GlyphAtlas {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
    allocation: Option<Allocation>,
    packer: GlyphPacker,
}

pub fn create_atlas(context: &Context) -> Result<GlyphAtlas, Box<dyn std::error::Error>> {
    let (image, view, allocation) = create_image(context, ATLAS_INITIAL_SIZE)?;
    let sampler = create_sampler(context)?;
    Ok(GlyphAtlas {
        image,
        view,
        sampler,
        allocation: Some(allocation),
        packer: GlyphPacker::default(),
    })
}

pub fn revision(atlas: &GlyphAtlas) -> u32 {
    atlas.packer.revision()
}

pub fn ensure_glyph(
    atlas: &mut GlyphAtlas,
    context: &Context,
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    cache_key: CacheKey,
) -> Option<GlyphAlloc> {
    let GlyphAtlas {
        image,
        view,
        allocation,
        packer,
        ..
    } = atlas;
    ensure_glyph_slot(
        packer,
        font_system,
        swash_cache,
        cache_key,
        &mut |work| match work {
            GlyphWork::Grow { previous, next } => {
                match grow(image, view, allocation, context, previous, next) {
                    Ok(()) => true,
                    Err(error) => {
                        tracing::error!("the glyph atlas did not grow: {error}");
                        false
                    }
                }
            }
            GlyphWork::Upload { x, y, glyph } => match upload(*image, context, x, y, glyph) {
                Ok(()) => true,
                Err(error) => {
                    tracing::error!("a glyph was not uploaded: {error}");
                    false
                }
            },
        },
    )
}

pub fn destroy_atlas(context: &Context, atlas: &mut GlyphAtlas) {
    unsafe {
        context.device.destroy_sampler(atlas.sampler, None);
        context.device.destroy_image_view(atlas.view, None);
        context.device.destroy_image(atlas.image, None);
    }
    release(context, atlas.allocation.take());
}

fn grow(
    atlas_image: &mut vk::Image,
    atlas_view: &mut vk::ImageView,
    atlas_allocation: &mut Option<Allocation>,
    context: &Context,
    previous: u32,
    size: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    unsafe { context.device.device_wait_idle()? };
    let (image, view, allocation) = create_image(context, size)?;
    copy_atlas(context, *atlas_image, image, previous)?;
    wait_for_sent_work(context)?;

    unsafe {
        context.device.destroy_image_view(*atlas_view, None);
        context.device.destroy_image(*atlas_image, None);
    }
    release(context, atlas_allocation.take());

    *atlas_image = image;
    *atlas_view = view;
    *atlas_allocation = Some(allocation);
    Ok(())
}

fn copy_atlas(
    context: &Context,
    source: vk::Image,
    destination: vk::Image,
    size: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let command_buffer = begin_one_shot(context)?;
    transition(
        context,
        command_buffer,
        source,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
    );
    transition(
        context,
        command_buffer,
        destination,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );
    let layers = vk::ImageSubresourceLayers {
        aspect_mask: vk::ImageAspectFlags::COLOR,
        mip_level: 0,
        base_array_layer: 0,
        layer_count: 1,
    };
    let region = vk::ImageCopy::default()
        .src_subresource(layers)
        .dst_subresource(layers)
        .extent(vk::Extent3D {
            width: size,
            height: size,
            depth: 1,
        });
    unsafe {
        context.device.cmd_copy_image(
            command_buffer,
            source,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            destination,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            std::slice::from_ref(&region),
        )
    };
    transition(
        context,
        command_buffer,
        destination,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    end_one_shot(context, command_buffer)
}

fn upload(
    atlas_image: vk::Image,
    context: &Context,
    x: u32,
    y: u32,
    glyph: &RasterizedGlyph,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut staging = create_buffer(
        context,
        "glyph staging",
        glyph.pixels.len() as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )?;
    write_buffer(&staging, &glyph.pixels);

    let command_buffer = begin_one_shot(context)?;
    transition(
        context,
        command_buffer,
        atlas_image,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );
    let region = vk::BufferImageCopy::default()
        .image_subresource(vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        })
        .image_offset(vk::Offset3D {
            x: x as i32,
            y: y as i32,
            z: 0,
        })
        .image_extent(vk::Extent3D {
            width: glyph.width,
            height: glyph.height,
            depth: 1,
        });
    unsafe {
        context.device.cmd_copy_buffer_to_image(
            command_buffer,
            staging.buffer,
            atlas_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            std::slice::from_ref(&region),
        )
    };
    transition(
        context,
        command_buffer,
        atlas_image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    end_one_shot(context, command_buffer)?;
    retire_buffer(context, &mut staging);
    Ok(())
}

fn create_image(
    context: &Context,
    size: u32,
) -> Result<(vk::Image, vk::ImageView, Allocation), Box<dyn std::error::Error>> {
    let create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(vk::Format::R8G8B8A8_UNORM)
        .extent(vk::Extent3D {
            width: size,
            height: size,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(
            vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_DST
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
            name: "glyph atlas",
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
        .format(vk::Format::R8G8B8A8_UNORM)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };

    let command_buffer = begin_one_shot(context)?;
    transition(
        context,
        command_buffer,
        image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    end_one_shot(context, command_buffer)?;

    Ok((image, view, allocation))
}

fn create_sampler(context: &Context) -> Result<vk::Sampler, Box<dyn std::error::Error>> {
    let create_info = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR);
    Ok(unsafe { context.device.create_sampler(&create_info, None)? })
}
