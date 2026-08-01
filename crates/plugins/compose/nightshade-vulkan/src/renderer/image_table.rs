use crate::renderer::context::Context;
use crate::renderer::memory::{
    Handover, begin_one_shot, begin_one_shot_on, create_buffer, end_one_shot_after,
    end_one_shot_on, hand_over_image, release, retire_buffer_on, sent_count, transition_range,
    write_buffer, write_buffer_at,
};
use ash::vk;
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};
use nightshade_render::data::decoded_image::TexelFormat;
use nightshade_render::data::texture_data::SamplerWrap;
use nightshade_render::queries::texture::SAMPLER_SLOT_COUNT;

pub const MAX_PAINT_IMAGES: u32 = 1024;

const SCENE_TEXTURE_CEILING: u32 = 16384;

pub fn scene_texture_capacity(context: &Context) -> u32 {
    let mut indexing = vk::PhysicalDeviceDescriptorIndexingProperties::default();
    let mut properties = vk::PhysicalDeviceProperties2::default().push_next(&mut indexing);
    unsafe {
        context
            .instance
            .get_physical_device_properties2(context.physical_device, &mut properties);
    }
    indexing
        .max_per_stage_descriptor_update_after_bind_sampled_images
        .clamp(1, SCENE_TEXTURE_CEILING)
}

struct Texture {
    image: vk::Image,
    view: vk::ImageView,
    allocation: Option<Allocation>,
    srgb: bool,
    mips: bool,
    format: TexelFormat,
}

pub struct ImageTable {
    pub layout: vk::DescriptorSetLayout,
    pub set: vk::DescriptorSet,
    pool: vk::DescriptorPool,
    samplers: Vec<vk::Sampler>,
    anisotropy: f32,
    fallback: Texture,
    textures: Vec<Option<Texture>>,
    capacity: u32,
    replaced: Vec<(u64, Texture)>,
    frames: u64,
}

pub struct ImageUpload<'pixels> {
    pub width: u32,
    pub height: u32,
    pub pixels: &'pixels [u8],
    pub srgb: bool,
    pub mips: bool,
    pub format: TexelFormat,
    pub levels: &'pixels [Vec<u8>],
}

pub fn create_image_table(
    context: &Context,
    capacity: u32,
    stages: vk::ShaderStageFlags,
) -> Result<ImageTable, Box<dyn std::error::Error>> {
    let anisotropy = clamp_anisotropy(context, 16.0);
    let samplers = create_samplers(context, anisotropy)?;
    let layout = create_layout(context, capacity, stages)?;
    let pool = create_pool(context, capacity)?;

    let layouts = [layout];
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let set = unsafe { context.device.allocate_descriptor_sets(&allocate_info)?[0] };

    let fallback = create_texture(context, 1, 1, vk::Format::R8G8B8A8_UNORM, 1)?;
    upload_pixels(
        context,
        &fallback,
        &ImageUpload {
            width: 1,
            height: 1,
            pixels: &[0, 0, 0, 0],
            srgb: false,
            mips: false,
            format: TexelFormat::Rgba8,
            levels: &[],
        },
        1,
    )?;

    let table = ImageTable {
        layout,
        set,
        pool,
        samplers,
        anisotropy,
        fallback,
        textures: (0..capacity).map(|_| None).collect(),
        capacity,
        replaced: Vec::new(),
        frames: 0,
    };
    for index in 0..capacity {
        write_image_descriptor(context, &table, index, table.fallback.view);
    }
    write_sampler_descriptors(context, &table);
    Ok(table)
}

pub fn upload_image(
    table: &mut ImageTable,
    context: &Context,
    index: u32,
    upload: ImageUpload<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    if index >= table.capacity {
        return Err(format!(
            "the texture table holds {} slots and slot {index} was asked for",
            table.capacity
        )
        .into());
    }
    if upload.width == 0 || upload.height == 0 || upload.pixels.is_empty() {
        return Ok(());
    }
    let format = device_format(context, upload.format, upload.srgb)?;
    let levels = if !upload.levels.is_empty() {
        1 + upload.levels.len() as u32
    } else if upload.mips && !upload.format.is_compressed() {
        mip_levels(upload.width, upload.height)
    } else {
        1
    };
    let mut texture = create_texture(context, upload.width, upload.height, format, levels)?;
    texture.srgb = upload.srgb;
    texture.mips = upload.mips;
    texture.format = upload.format;
    upload_pixels(context, &texture, &upload, levels)?;
    write_image_descriptor(context, table, index, texture.view);

    if let Some(previous) = table.textures[index as usize].replace(texture) {
        let taken_over_on = table.frames;
        table.replaced.push((taken_over_on, previous));
    }
    Ok(())
}

pub fn reload_image(
    table: &mut ImageTable,
    context: &Context,
    index: u32,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(holding) = table.textures.get(index as usize).and_then(Option::as_ref) else {
        return Ok(());
    };
    if holding.format.is_compressed() {
        return Err("a block compressed texture cannot take plain pixels".into());
    }
    let upload = ImageUpload {
        width,
        height,
        pixels,
        srgb: holding.srgb,
        mips: holding.mips,
        format: TexelFormat::Rgba8,
        levels: &[],
    };
    upload_image(table, context, index, upload)
}

fn device_format(
    context: &Context,
    format: TexelFormat,
    srgb: bool,
) -> Result<vk::Format, Box<dyn std::error::Error>> {
    let bc = context.compression.bc;
    let astc = context.compression.astc;
    let chosen = match (format, srgb) {
        (TexelFormat::Rgba8, false) => vk::Format::R8G8B8A8_UNORM,
        (TexelFormat::Rgba8, true) => vk::Format::R8G8B8A8_SRGB,
        (TexelFormat::Bc1, false) if bc => vk::Format::BC1_RGBA_UNORM_BLOCK,
        (TexelFormat::Bc1, true) if bc => vk::Format::BC1_RGBA_SRGB_BLOCK,
        (TexelFormat::Bc3, false) if bc => vk::Format::BC3_UNORM_BLOCK,
        (TexelFormat::Bc3, true) if bc => vk::Format::BC3_SRGB_BLOCK,
        (TexelFormat::Bc4, _) if bc => vk::Format::BC4_UNORM_BLOCK,
        (TexelFormat::Bc5, _) if bc => vk::Format::BC5_UNORM_BLOCK,
        (TexelFormat::Bc7, false) if bc => vk::Format::BC7_UNORM_BLOCK,
        (TexelFormat::Bc7, true) if bc => vk::Format::BC7_SRGB_BLOCK,
        (TexelFormat::Astc4x4, false) if astc => vk::Format::ASTC_4X4_UNORM_BLOCK,
        (TexelFormat::Astc4x4, true) if astc => vk::Format::ASTC_4X4_SRGB_BLOCK,
        _ => return Err(format!("the device does not read {format:?} textures").into()),
    };
    Ok(chosen)
}

pub fn free_image(table: &mut ImageTable, context: &Context, index: u32) {
    if index >= table.capacity {
        return;
    }
    let Some(previous) = table.textures[index as usize].take() else {
        return;
    };
    write_image_descriptor(context, table, index, table.fallback.view);
    let taken_over_on = table.frames;
    table.replaced.push((taken_over_on, previous));
}

pub fn retire_replaced(table: &mut ImageTable, context: &Context, frames_in_flight: u64) {
    table.frames += 1;
    let reached = table.frames;
    let mut index = 0;
    while index < table.replaced.len() {
        if reached.saturating_sub(table.replaced[index].0) > frames_in_flight {
            let (_, mut texture) = table.replaced.remove(index);
            destroy_texture(context, &mut texture);
        } else {
            index += 1;
        }
    }
}

pub fn destroy_image_table(context: &Context, table: &mut ImageTable) {
    for (_, texture) in &mut table.replaced {
        destroy_texture(context, texture);
    }
    table.replaced.clear();
    for slot in &mut table.textures {
        if let Some(texture) = slot.as_mut() {
            destroy_texture(context, texture);
        }
    }
    table.textures.clear();
    destroy_texture(context, &mut table.fallback);
    unsafe {
        for sampler in table.samplers.drain(..) {
            context.device.destroy_sampler(sampler, None);
        }
        context.device.destroy_descriptor_pool(table.pool, None);
        context
            .device
            .destroy_descriptor_set_layout(table.layout, None);
    }
}

fn mip_levels(width: u32, height: u32) -> u32 {
    32 - width.max(height).leading_zeros()
}

fn write_image_descriptor(context: &Context, table: &ImageTable, index: u32, view: vk::ImageView) {
    let info = vk::DescriptorImageInfo::default()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(view);
    let write = vk::WriteDescriptorSet::default()
        .dst_set(table.set)
        .dst_binding(0)
        .dst_array_element(index)
        .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
        .image_info(std::slice::from_ref(&info));
    unsafe {
        context
            .device
            .update_descriptor_sets(std::slice::from_ref(&write), &[])
    };
}

fn write_sampler_descriptors(context: &Context, table: &ImageTable) {
    let infos: Vec<vk::DescriptorImageInfo> = table
        .samplers
        .iter()
        .map(|sampler| vk::DescriptorImageInfo::default().sampler(*sampler))
        .collect();
    let write = vk::WriteDescriptorSet::default()
        .dst_set(table.set)
        .dst_binding(1)
        .dst_array_element(0)
        .descriptor_type(vk::DescriptorType::SAMPLER)
        .image_info(&infos);
    unsafe {
        context
            .device
            .update_descriptor_sets(std::slice::from_ref(&write), &[])
    };
}

fn create_layout(
    context: &Context,
    capacity: u32,
    stages: vk::ShaderStageFlags,
) -> Result<vk::DescriptorSetLayout, Box<dyn std::error::Error>> {
    let bindings = [
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
            .descriptor_count(capacity)
            .stage_flags(stages),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::SAMPLER)
            .descriptor_count(SAMPLER_SLOT_COUNT)
            .stage_flags(stages),
    ];
    let flags = [
        vk::DescriptorBindingFlags::PARTIALLY_BOUND
            | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
            | vk::DescriptorBindingFlags::UPDATE_UNUSED_WHILE_PENDING,
        vk::DescriptorBindingFlags::PARTIALLY_BOUND,
    ];
    let mut binding_flags =
        vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&flags);
    let create_info = vk::DescriptorSetLayoutCreateInfo::default()
        .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
        .bindings(&bindings)
        .push_next(&mut binding_flags);
    Ok(unsafe {
        context
            .device
            .create_descriptor_set_layout(&create_info, None)?
    })
}

fn create_pool(
    context: &Context,
    capacity: u32,
) -> Result<vk::DescriptorPool, Box<dyn std::error::Error>> {
    let sizes = [
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::SAMPLED_IMAGE)
            .descriptor_count(capacity),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::SAMPLER)
            .descriptor_count(SAMPLER_SLOT_COUNT),
    ];
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
        .pool_sizes(&sizes)
        .max_sets(1);
    Ok(unsafe { context.device.create_descriptor_pool(&create_info, None)? })
}

fn clamp_anisotropy(context: &Context, wanted: f32) -> f32 {
    let limits = unsafe {
        context
            .instance
            .get_physical_device_properties(context.physical_device)
            .limits
    };
    wanted.clamp(1.0, limits.max_sampler_anisotropy)
}

pub fn set_anisotropy(
    table: &mut ImageTable,
    context: &Context,
    wanted: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let anisotropy = clamp_anisotropy(context, wanted);
    if (anisotropy - table.anisotropy).abs() < 0.5 {
        return Ok(());
    }
    let samplers = create_samplers(context, anisotropy)?;
    unsafe { context.device.device_wait_idle()? };
    for sampler in table.samplers.drain(..) {
        unsafe { context.device.destroy_sampler(sampler, None) };
    }
    table.samplers = samplers;
    table.anisotropy = anisotropy;
    write_sampler_descriptors(context, table);
    Ok(())
}

fn create_samplers(
    context: &Context,
    anisotropy: f32,
) -> Result<Vec<vk::Sampler>, Box<dyn std::error::Error>> {
    let wraps = [
        SamplerWrap::Repeat,
        SamplerWrap::MirroredRepeat,
        SamplerWrap::ClampToEdge,
    ];
    let mut samplers = Vec::with_capacity(SAMPLER_SLOT_COUNT as usize);
    for filter in [vk::Filter::LINEAR, vk::Filter::NEAREST] {
        let anisotropic = filter == vk::Filter::LINEAR && anisotropy > 1.0;
        for wrap_u in wraps {
            for wrap_v in wraps {
                let create_info = vk::SamplerCreateInfo::default()
                    .mag_filter(filter)
                    .min_filter(filter)
                    .address_mode_u(address_mode(wrap_u))
                    .address_mode_v(address_mode(wrap_v))
                    .address_mode_w(vk::SamplerAddressMode::REPEAT)
                    .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                    .max_lod(vk::LOD_CLAMP_NONE)
                    .anisotropy_enable(anisotropic)
                    .max_anisotropy(if anisotropic { anisotropy } else { 1.0 });
                samplers.push(unsafe { context.device.create_sampler(&create_info, None)? });
            }
        }
    }
    Ok(samplers)
}

fn address_mode(wrap: SamplerWrap) -> vk::SamplerAddressMode {
    match wrap {
        SamplerWrap::Repeat => vk::SamplerAddressMode::REPEAT,
        SamplerWrap::MirroredRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
        SamplerWrap::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
    }
}

fn create_texture(
    context: &Context,
    width: u32,
    height: u32,
    format: vk::Format,
    levels: u32,
) -> Result<Texture, Box<dyn std::error::Error>> {
    let create_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(levels)
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
            name: "bindless texture",
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
            level_count: levels,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = unsafe { context.device.create_image_view(&view_info, None)? };

    Ok(Texture {
        image,
        view,
        allocation: Some(allocation),
        srgb: false,
        mips: false,
        format: TexelFormat::Rgba8,
    })
}

fn upload_pixels(
    context: &Context,
    texture: &Texture,
    upload: &ImageUpload<'_>,
    levels: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let width = upload.width;
    let height = upload.height;
    let carried = 1 + upload.levels.len() as u32;
    let staged: usize =
        upload.pixels.len() + upload.levels.iter().map(|level| level.len()).sum::<usize>();
    let mut staging = create_buffer(
        context,
        "texture staging",
        staged as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
    )?;
    write_buffer(&staging, upload.pixels);
    let mut offsets = vec![0u64];
    let mut written = upload.pixels.len() as u64;
    for level in upload.levels {
        write_buffer_at(&staging, written, level);
        offsets.push(written);
        written += level.len() as u64;
    }

    let mover = context.upload_lane();
    let command_buffer = begin_one_shot_on(context, mover)?;
    transition_range(
        context,
        command_buffer,
        texture.image,
        vk::ImageAspectFlags::COLOR,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        0,
        levels,
    );
    let regions: Vec<vk::BufferImageCopy> = offsets
        .iter()
        .enumerate()
        .map(|(level, offset)| {
            vk::BufferImageCopy::default()
                .buffer_offset(*offset)
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: level as u32,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image_extent(vk::Extent3D {
                    width: (width >> level).max(1),
                    height: (height >> level).max(1),
                    depth: 1,
                })
        })
        .collect();
    unsafe {
        context.device.cmd_copy_buffer_to_image(
            command_buffer,
            staging.buffer,
            texture.image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &regions,
        )
    };
    let build = levels > carried;
    let drawer = &context.graphics;
    if mover.family == drawer.family {
        finish_texture(
            context,
            command_buffer,
            texture,
            width,
            height,
            levels,
            build,
        );
        end_one_shot_on(context, mover, command_buffer)?;
        retire_buffer_on(context, mover, &mut staging);
        return Ok(());
    }
    hand_over_image(
        context,
        command_buffer,
        texture.image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        levels,
        mover,
        drawer,
        Handover::Release,
    );
    end_one_shot_on(context, mover, command_buffer)?;
    retire_buffer_on(context, mover, &mut staging);
    let landed = sent_count(mover);

    let command_buffer = begin_one_shot(context)?;
    hand_over_image(
        context,
        command_buffer,
        texture.image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        levels,
        mover,
        drawer,
        Handover::Acquire,
    );
    finish_texture(
        context,
        command_buffer,
        texture,
        width,
        height,
        levels,
        build,
    );
    end_one_shot_after(context, drawer, command_buffer, mover, landed)
}

fn finish_texture(
    context: &Context,
    command_buffer: vk::CommandBuffer,
    texture: &Texture,
    width: u32,
    height: u32,
    levels: u32,
    build: bool,
) {
    if build {
        generate_mips(
            context,
            command_buffer,
            texture.image,
            width,
            height,
            levels,
        );
    }
    transition_range(
        context,
        command_buffer,
        texture.image,
        vk::ImageAspectFlags::COLOR,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        0,
        levels,
    );
}

fn generate_mips(
    context: &Context,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    width: u32,
    height: u32,
    levels: u32,
) {
    let mut source_width = width;
    let mut source_height = height;
    for level in 1..levels {
        let target_width = (source_width / 2).max(1);
        let target_height = (source_height / 2).max(1);
        transition_range(
            context,
            command_buffer,
            image,
            vk::ImageAspectFlags::COLOR,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            level - 1,
            1,
        );
        let blit = vk::ImageBlit::default()
            .src_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: level - 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .src_offsets([
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: source_width as i32,
                    y: source_height as i32,
                    z: 1,
                },
            ])
            .dst_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: level,
                base_array_layer: 0,
                layer_count: 1,
            })
            .dst_offsets([
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: target_width as i32,
                    y: target_height as i32,
                    z: 1,
                },
            ]);
        unsafe {
            context.device.cmd_blit_image(
                command_buffer,
                image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(&blit),
                vk::Filter::LINEAR,
            )
        };
        transition_range(
            context,
            command_buffer,
            image,
            vk::ImageAspectFlags::COLOR,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            level - 1,
            1,
        );
        source_width = target_width;
        source_height = target_height;
    }
}

fn destroy_texture(context: &Context, texture: &mut Texture) {
    unsafe {
        context.device.destroy_image_view(texture.view, None);
        context.device.destroy_image(texture.image, None);
    }
    release(context, texture.allocation.take());
}
