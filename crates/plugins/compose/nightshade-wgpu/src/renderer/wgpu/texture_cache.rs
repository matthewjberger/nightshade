use nightshade_render::data::asset_id::TextureId;
use nightshade_render::data::generational_registry::*;
use nightshade_render::data::texture_data::{
    SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage,
};
use nightshade_render::queries::image::downsample_rgba8_box;
pub use nightshade_render::resources::texture_cache::TextureCache;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub fn texture_usage_format(usage: TextureUsage) -> wgpu::TextureFormat {
    match usage {
        TextureUsage::Color => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureUsage::Linear => wgpu::TextureFormat::Rgba8Unorm,
    }
}

pub fn sampler_wrap_address_mode(wrap: SamplerWrap) -> wgpu::AddressMode {
    match wrap {
        SamplerWrap::Repeat => wgpu::AddressMode::Repeat,
        SamplerWrap::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
        SamplerWrap::ClampToEdge => wgpu::AddressMode::ClampToEdge,
    }
}

pub fn sampler_filter_mode(filter: SamplerFilter) -> wgpu::FilterMode {
    match filter {
        SamplerFilter::Nearest => wgpu::FilterMode::Nearest,
        SamplerFilter::Linear => wgpu::FilterMode::Linear,
    }
}

pub fn sampler_filter_mipmap_mode(filter: SamplerFilter) -> wgpu::MipmapFilterMode {
    match filter {
        SamplerFilter::Nearest => wgpu::MipmapFilterMode::Nearest,
        SamplerFilter::Linear => wgpu::MipmapFilterMode::Linear,
    }
}

pub struct TextureEntry {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

#[derive(Default)]
pub struct TextureStore {
    entries: HashMap<String, TextureEntry>,
}

pub fn texture_store_get<'a>(store: &'a TextureStore, name: &str) -> Option<&'a TextureEntry> {
    store.entries.get(name)
}

pub fn texture_store_contains(store: &TextureStore, name: &str) -> bool {
    store.entries.contains_key(name)
}

pub fn texture_store_names(store: &TextureStore) -> impl Iterator<Item = &String> {
    store.entries.keys()
}

pub fn texture_store_insert(store: &mut TextureStore, name: String, entry: TextureEntry) {
    store.entries.insert(name, entry);
}

pub fn texture_store_remove(store: &mut TextureStore, name: &str) -> bool {
    store.entries.remove(name).is_some()
}

#[derive(Debug, Clone, Copy)]
pub struct TextureUploadSpec {
    pub format: wgpu::TextureFormat,
    pub sampler: SamplerSettings,
}

pub struct TextureUploadRequest<'a> {
    pub name: String,
    pub rgba_data: &'a [u8],
    pub dimensions: (u32, u32),
    pub spec: TextureUploadSpec,
}

pub fn texture_cache_load_from_raw_rgba_with_format(
    cache: &mut TextureCache,
    store: &mut TextureStore,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mip_generator: &super::mip_generator::MipGenerator,
    request: TextureUploadRequest<'_>,
) -> Result<TextureId, String> {
    let TextureUploadRequest {
        name,
        rgba_data,
        dimensions,
        spec,
    } = request;
    let format = spec.format;
    let sampler_settings = spec.sampler;
    if texture_store_contains(store, &name)
        && let Some((index, generation)) = registry_lookup_index(&cache.registry, &name)
    {
        return Ok(TextureId::new(index, generation));
    }

    let (width, height) = dimensions;
    let largest = width.max(height).max(1);
    let mip_level_count = (largest as f32).log2().floor() as u32 + 1;

    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let supports_gpu_mips = matches!(
        format,
        wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Rgba8Unorm
    ) && mip_level_count > 1;

    let mut usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
    if supports_gpu_mips {
        usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
    }

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&name),
        size,
        mip_level_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        rgba_data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        size,
    );

    if supports_gpu_mips {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Texture Cache Mip Gen"),
        });
        mip_generator.generate_mips(device, &mut encoder, &texture, 0);
        queue.submit(std::iter::once(encoder.finish()));
    } else if mip_level_count > 1 {
        let is_srgb = matches!(
            format,
            wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8UnormSrgb
        );
        let mut current = rgba_data.to_vec();
        let mut current_width = width;
        let mut current_height = height;
        for mip in 1..mip_level_count {
            let (downsampled, new_width, new_height) =
                downsample_rgba8_box(&current, current_width, current_height, is_srgb);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: mip,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &downsampled,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * new_width),
                    rows_per_image: Some(new_height),
                },
                wgpu::Extent3d {
                    width: new_width,
                    height: new_height,
                    depth_or_array_layers: 1,
                },
            );
            current = downsampled;
            current_width = new_width;
            current_height = new_height;
        }
    }

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mipmap_filter = crate::renderer::wgpu::texture_cache::sampler_filter_mipmap_mode(
        sampler_settings.mipmap_filter,
    );
    let min_filter =
        crate::renderer::wgpu::texture_cache::sampler_filter_mode(sampler_settings.min_filter);
    let mag_filter =
        crate::renderer::wgpu::texture_cache::sampler_filter_mode(sampler_settings.mag_filter);
    let supports_anisotropy = matches!(min_filter, wgpu::FilterMode::Linear)
        && matches!(mag_filter, wgpu::FilterMode::Linear)
        && matches!(mipmap_filter, wgpu::MipmapFilterMode::Linear);

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(&format!("{} Sampler", name)),
        address_mode_u: crate::renderer::wgpu::texture_cache::sampler_wrap_address_mode(
            sampler_settings.wrap_u,
        ),
        address_mode_v: crate::renderer::wgpu::texture_cache::sampler_wrap_address_mode(
            sampler_settings.wrap_v,
        ),
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter,
        min_filter,
        mipmap_filter,
        anisotropy_clamp: if supports_anisotropy { 16 } else { 1 },
        ..Default::default()
    });

    let entry = TextureEntry {
        texture,
        view,
        sampler,
    };

    let (index, generation) = registry_insert(&mut cache.registry, name.clone(), ());
    texture_store_insert(store, name, entry);

    Ok(TextureId::new(index, generation))
}

pub enum TextureReloadResult {
    UpdatedInPlace,
    Recreated,
    NotFound,
}

pub fn texture_cache_reload(
    cache: &TextureCache,
    store: &mut TextureStore,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    name: &str,
    rgba_data: &[u8],
    dimensions: (u32, u32),
) -> TextureReloadResult {
    let (width, height) = dimensions;
    if !cache.registry.name_to_index.contains_key(name) {
        tracing::warn!("Texture reload: '{}' not found in cache", name);
        return TextureReloadResult::NotFound;
    }
    let Some(entry) = texture_store_get(store, name) else {
        tracing::warn!("Texture reload: '{}' entry is empty", name);
        return TextureReloadResult::NotFound;
    };

    let existing_size = entry.texture.size();
    if existing_size.width == width && existing_size.height == height {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &entry.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        TextureReloadResult::UpdatedInPlace
    } else {
        let existing_format = entry.texture.format();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some(name),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: existing_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            rgba_data,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{} Sampler", name)),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        texture_store_insert(
            store,
            name.to_string(),
            TextureEntry {
                texture,
                view,
                sampler,
            },
        );
        TextureReloadResult::Recreated
    }
}
