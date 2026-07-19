use crate::asset_id::TextureId;
use crate::generational_registry::*;
use crate::texture_data::{SamplerFilter, SamplerSettings, SamplerWrap, TextureUsage};
use nightshade_ecs::Entity;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

impl TextureUsage {
    /// The wgpu texture format for this usage, sRGB for color and linear otherwise.
    pub fn wgpu_format(self) -> wgpu::TextureFormat {
        match self {
            Self::Color => wgpu::TextureFormat::Rgba8UnormSrgb,
            Self::Linear => wgpu::TextureFormat::Rgba8Unorm,
        }
    }
}

impl SamplerWrap {
    /// The wgpu address mode for this wrap setting.
    pub fn wgpu_address_mode(self) -> wgpu::AddressMode {
        match self {
            Self::Repeat => wgpu::AddressMode::Repeat,
            Self::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
            Self::ClampToEdge => wgpu::AddressMode::ClampToEdge,
        }
    }
}

impl SamplerFilter {
    /// The wgpu min/mag filter mode for this filter setting.
    pub fn wgpu_filter_mode(self) -> wgpu::FilterMode {
        match self {
            Self::Nearest => wgpu::FilterMode::Nearest,
            Self::Linear => wgpu::FilterMode::Linear,
        }
    }

    /// The wgpu mipmap filter mode for this filter setting.
    pub fn wgpu_mipmap_filter_mode(self) -> wgpu::MipmapFilterMode {
        match self {
            Self::Nearest => wgpu::MipmapFilterMode::Nearest,
            Self::Linear => wgpu::MipmapFilterMode::Linear,
        }
    }
}

/// Who holds a set of texture references. Entity owners split the material's
/// own textures from prefab variant textures so a material swap replaces one
/// set without disturbing the other; named owners cover keep-alives that are
/// not tied to an entity.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TextureOwner {
    /// The textures of an entity's own material.
    EntityMaterial(Entity),
    /// The prefab variant textures of an entity.
    EntityVariants(Entity),
    /// A named keep-alive not tied to any entity.
    Named(String),
}

/// CPU-side texture registry: reference-counted, owner-tracked id and name
/// bookkeeping. Holds no GPU handles. The wgpu textures live in the
/// renderer-owned [`TextureStore`], keyed by the same names. The host owns this
/// (it allocates ids, refcounts, and drives eviction); the renderer owns the
/// store, so nothing GPU-able sits on the host's side of the boundary.
#[derive(Default)]
pub struct TextureCache {
    pub(crate) registry: GenerationalRegistry<()>,
    pub(crate) pending_references: HashMap<String, usize>,
    pub(crate) protected_names: std::collections::HashSet<String>,
    pub(crate) owners: HashMap<TextureOwner, Vec<String>>,
}

/// A GPU texture together with its default view and sampler.
pub struct TextureEntry {
    /// The GPU texture resource.
    pub texture: wgpu::Texture,
    /// Default view over all mip levels.
    pub view: wgpu::TextureView,
    /// Sampler configured from the upload's settings.
    pub sampler: wgpu::Sampler,
}

/// The renderer-owned store of live GPU textures, keyed by name. Persists on the
/// [`WgpuRenderer`](crate::wgpu::WgpuRenderer) across frames and is swapped into
/// the frame's [`RenderInputs`](crate::wgpu::render_configs::RenderInputs) so the
/// passes can sample it, never touching the host. The CPU id/refcount
/// bookkeeping lives in [`TextureCache`].
#[derive(Default)]
pub struct TextureStore {
    entries: HashMap<String, TextureEntry>,
}

/// The GPU texture entry stored under `name`, if uploaded.
pub fn texture_store_get<'a>(store: &'a TextureStore, name: &str) -> Option<&'a TextureEntry> {
    store.entries.get(name)
}

/// True when a GPU texture is stored under `name`.
pub fn texture_store_contains(store: &TextureStore, name: &str) -> bool {
    store.entries.contains_key(name)
}

/// The names of every stored GPU texture.
pub fn texture_store_names(store: &TextureStore) -> impl Iterator<Item = &String> {
    store.entries.keys()
}

pub(crate) fn texture_store_insert(store: &mut TextureStore, name: String, entry: TextureEntry) {
    store.entries.insert(name, entry);
}

pub(crate) fn texture_store_remove(store: &mut TextureStore, name: &str) -> bool {
    store.entries.remove(name).is_some()
}

/// Format and sampler settings for a texture upload.
#[derive(Debug, Clone, Copy)]
pub struct TextureUploadSpec {
    /// GPU format the texture is created with.
    pub format: wgpu::TextureFormat,
    /// Sampler settings used to build the entry's sampler.
    pub sampler: SamplerSettings,
}

/// A single raw-RGBA texture upload into the cache.
pub struct TextureUploadRequest<'a> {
    /// Registry name the texture is stored under.
    pub name: String,
    /// Tightly packed RGBA8 pixel data for mip level 0.
    pub rgba_data: &'a [u8],
    /// Width and height in pixels.
    pub dimensions: (u32, u32),
    /// Format and sampler settings for the upload.
    pub spec: TextureUploadSpec,
}

/// Uploads raw RGBA8 pixels as a texture, generating a mip chain, and returns
/// its stable id. Returns the existing id when the name is already filled.
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
    if let Some((index, generation)) = registry_lookup_index(&cache.registry, &name)
        && registry_is_filled(&cache.registry, index)
    {
        return Ok(TextureId::new(index, generation));
    }

    let pending_refs = cache.pending_references.remove(&name).unwrap_or(0);

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

    let mipmap_filter = sampler_settings.mipmap_filter.wgpu_mipmap_filter_mode();
    let min_filter = sampler_settings.min_filter.wgpu_filter_mode();
    let mag_filter = sampler_settings.mag_filter.wgpu_filter_mode();
    let supports_anisotropy = matches!(min_filter, wgpu::FilterMode::Linear)
        && matches!(mag_filter, wgpu::FilterMode::Linear)
        && matches!(mipmap_filter, wgpu::MipmapFilterMode::Linear);

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(&format!("{} Sampler", name)),
        address_mode_u: sampler_settings.wrap_u.wgpu_address_mode(),
        address_mode_v: sampler_settings.wrap_v.wgpu_address_mode(),
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
    cache.registry.reference_counts[index as usize] += pending_refs;
    texture_store_insert(store, name, entry);

    Ok(TextureId::new(index, generation))
}

/// Reserves a stable id for `name` before its texture is uploaded, so callers
/// can reference it ahead of the pixel data arriving.
pub fn texture_cache_reserve_id(cache: &mut TextureCache, name: String) -> TextureId {
    let (index, generation) = registry_reserve_name(&mut cache.registry, name);
    TextureId::new(index, generation)
}

fn increment_reference(cache: &mut TextureCache, name: &str) {
    if let Some(&index) = cache.registry.name_to_index.get(name) {
        registry_add_reference(&mut cache.registry, index);
    } else {
        *cache
            .pending_references
            .entry(name.to_string())
            .or_insert(0) += 1;
    }
}

fn decrement_reference(cache: &mut TextureCache, name: &str) {
    if let Some(&index) = cache.registry.name_to_index.get(name) {
        registry_remove_reference(&mut cache.registry, index);
    } else if let Some(count) = cache.pending_references.get_mut(name) {
        *count = count.saturating_sub(1);
        if *count == 0 {
            cache.pending_references.remove(name);
        }
    }
}

/// Records `names` as held by `owner`, incrementing each texture's reference
/// count. Any set previously recorded for the same owner is released first,
/// so re-acquiring on a material swap is a single call and reference counts
/// never drift from what the ledger recorded.
pub fn texture_cache_acquire(cache: &mut TextureCache, owner: TextureOwner, names: Vec<String>) {
    if let Some(previous) = cache.owners.remove(&owner) {
        for name in &previous {
            decrement_reference(cache, name);
        }
    }
    for name in &names {
        increment_reference(cache, name);
    }
    if !names.is_empty() {
        cache.owners.insert(owner, names);
    }
}

/// Releases exactly the set recorded for `owner`. Unknown owners are a no-op,
/// so double releases and releases for entities that never acquired cannot
/// underflow a count.
pub fn texture_cache_release(cache: &mut TextureCache, owner: &TextureOwner) {
    if let Some(names) = cache.owners.remove(owner) {
        for name in &names {
            decrement_reference(cache, name);
        }
    }
}

/// Releases every entity-scoped set recorded for `entity`: the material's
/// textures and any prefab variant textures.
pub fn texture_cache_release_entity(cache: &mut TextureCache, entity: Entity) {
    texture_cache_release(cache, &TextureOwner::EntityMaterial(entity));
    texture_cache_release(cache, &TextureOwner::EntityVariants(entity));
}

/// Releases every entity-scoped set whose entity fails `keep`, covering
/// owners that died without their release running, such as a world restore
/// replacing the entity population wholesale. Named keep-alives are never
/// touched.
pub fn texture_cache_release_dead_entities(
    cache: &mut TextureCache,
    keep: impl Fn(Entity) -> bool,
) {
    let dead: Vec<TextureOwner> = cache
        .owners
        .keys()
        .filter(|owner| match owner {
            TextureOwner::EntityMaterial(entity) | TextureOwner::EntityVariants(entity) => {
                !keep(*entity)
            }
            TextureOwner::Named(_) => false,
        })
        .cloned()
        .collect();
    for owner in dead {
        texture_cache_release(cache, &owner);
    }
}

/// Resolves a texture name to its stable id, if the name has been reserved
/// or uploaded.
pub fn texture_cache_texture_id(cache: &TextureCache, name: &str) -> Option<TextureId> {
    registry_lookup_index(&cache.registry, name)
        .map(|(index, generation)| TextureId::new(index, generation))
}

/// Resolves a texture id's slot index back to its registered name.
pub fn texture_cache_texture_name(cache: &TextureCache, index: u32) -> Option<&str> {
    registry_name_for(&cache.registry, index)
}

/// Evicts every entry with a zero reference count, except protected names, and
/// returns the removed names.
pub fn texture_cache_remove_unused(cache: &mut TextureCache) -> Vec<String> {
    let mut removed = Vec::new();
    for index in 0..cache.registry.entries.len() {
        if cache.registry.reference_counts[index] == 0 && cache.registry.entries[index].is_some() {
            if let Some(name) = cache.registry.index_to_name[index].as_ref()
                && cache.protected_names.contains(name)
            {
                continue;
            }
            if let Some(name) = cache.registry.index_to_name[index].take() {
                cache.registry.name_to_index.remove(&name);
                removed.push(name);
            }
            cache.registry.entries[index] = None;
            cache.registry.free_indices.push(index as u32);
        }
    }
    removed
}

/// Marks `name` as protected so [`texture_cache_remove_unused`] never evicts it.
pub fn texture_cache_protect(cache: &mut TextureCache, name: String) {
    cache.protected_names.insert(name);
}

/// Outcome of [`texture_cache_reload`].
pub enum TextureReloadResult {
    /// The pixels were written into the existing texture at the same size.
    UpdatedInPlace,
    /// The texture was recreated because the new dimensions differ.
    Recreated,
    /// No entry with the given name exists.
    NotFound,
}

/// Replaces a registered texture's pixels, writing in place when the size is
/// unchanged and recreating the texture when it differs.
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

fn downsample_rgba8_box(
    src: &[u8],
    src_width: u32,
    src_height: u32,
    is_srgb: bool,
) -> (Vec<u8>, u32, u32) {
    let dst_width = (src_width / 2).max(1);
    let dst_height = (src_height / 2).max(1);
    let mut dst = vec![0u8; (dst_width * dst_height * 4) as usize];

    for y in 0..dst_height {
        for x in 0..dst_width {
            let dst_index = ((y * dst_width + x) * 4) as usize;
            let mut sum = [0.0_f32; 4];
            let mut count = 0.0_f32;

            for dy in 0..2 {
                for dx in 0..2 {
                    let sx = (x * 2 + dx).min(src_width.saturating_sub(1));
                    let sy = (y * 2 + dy).min(src_height.saturating_sub(1));
                    let src_index = ((sy * src_width + sx) * 4) as usize;

                    if is_srgb {
                        sum[0] += srgb_byte_to_linear(src[src_index]);
                        sum[1] += srgb_byte_to_linear(src[src_index + 1]);
                        sum[2] += srgb_byte_to_linear(src[src_index + 2]);
                    } else {
                        sum[0] += src[src_index] as f32 / 255.0;
                        sum[1] += src[src_index + 1] as f32 / 255.0;
                        sum[2] += src[src_index + 2] as f32 / 255.0;
                    }
                    sum[3] += src[src_index + 3] as f32 / 255.0;
                    count += 1.0;
                }
            }

            sum[0] /= count;
            sum[1] /= count;
            sum[2] /= count;
            sum[3] /= count;

            if is_srgb {
                dst[dst_index] = linear_to_srgb_byte(sum[0]);
                dst[dst_index + 1] = linear_to_srgb_byte(sum[1]);
                dst[dst_index + 2] = linear_to_srgb_byte(sum[2]);
            } else {
                dst[dst_index] = (sum[0] * 255.0).round().clamp(0.0, 255.0) as u8;
                dst[dst_index + 1] = (sum[1] * 255.0).round().clamp(0.0, 255.0) as u8;
                dst[dst_index + 2] = (sum[2] * 255.0).round().clamp(0.0, 255.0) as u8;
            }
            dst[dst_index + 3] = (sum[3] * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }

    (dst, dst_width, dst_height)
}

fn srgb_byte_to_linear(value: u8) -> f32 {
    let normalized = value as f32 / 255.0;
    if normalized <= 0.04045 {
        normalized / 12.92
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb_byte(value: f32) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    let encoded = if clamped <= 0.0031308 {
        clamped * 12.92
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round().clamp(0.0, 255.0) as u8
}
