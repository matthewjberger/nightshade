//! Bindless material texture arrays on the GPU.
//!
//! Material textures are packed into two `D2Array` textures (one sRGB, one
//! linear) sampled by layer index, or into a bindless array of individual
//! textures when the device supports it. Layer slots are reserved up front,
//! streamed in place, and returned to a free list on eviction.

use crate::texture_data::{SamplerWrap, TextureUsage};
use std::collections::HashMap;

const DEFAULT_LAYER_SIZE: u32 = 1024;
const DEFAULT_MAX_LAYERS: u32 = 256;

/// Selects the bindless texture path and its upper texture bound.
#[derive(Copy, Clone, Debug, Default)]
pub struct BindlessConfig {
    /// Whether the bindless path is used instead of the fixed array textures.
    pub enabled: bool,
    /// Maximum number of individual textures the bindless array holds.
    pub max_textures: u32,
}

fn wrap_code(wrap: SamplerWrap) -> u32 {
    match wrap {
        SamplerWrap::Repeat => 0,
        SamplerWrap::MirroredRepeat => 1,
        SamplerWrap::ClampToEdge => 2,
    }
}

/// Packs a layer index with its two wrap modes into a single `u32`: layer in
/// the low 16 bits, `wrap_u` at bit 16, `wrap_v` at bit 18.
pub fn pack_layer(layer: u32, wrap_u: SamplerWrap, wrap_v: SamplerWrap) -> u32 {
    (layer & 0xFFFFu32) | (wrap_code(wrap_u) << 16) | (wrap_code(wrap_v) << 18)
}

/// Where a named material texture lives in the arrays and how it wraps.
#[derive(Copy, Clone, Debug)]
pub struct MaterialTextureLayer {
    /// Color space bucket that decides which array texture holds the layer.
    pub usage: TextureUsage,
    /// Array layer index, or bindless slot index in the bindless path.
    pub layer: u32,
    /// Horizontal sampler wrap mode.
    pub wrap_u: SamplerWrap,
    /// Vertical sampler wrap mode.
    pub wrap_v: SamplerWrap,
}

impl MaterialTextureLayer {
    /// Layer index and wrap modes packed into one `u32` for the shader.
    pub fn packed(&self) -> u32 {
        pack_layer(self.layer, self.wrap_u, self.wrap_v)
    }
}

/// A pending texture upload identified by name with its pixels and sampling.
pub struct MaterialTextureUpload<'a> {
    /// Key the layer is registered under and reused across reservations.
    pub name: String,
    /// Tightly packed RGBA8 source pixels.
    pub rgba_data: &'a [u8],
    /// Source width in pixels.
    pub width: u32,
    /// Source height in pixels.
    pub height: u32,
    /// Color space bucket for the destination array.
    pub usage: TextureUsage,
    /// Horizontal sampler wrap mode.
    pub wrap_u: SamplerWrap,
    /// Vertical sampler wrap mode.
    pub wrap_v: SamplerWrap,
}

/// Default anisotropic filtering clamp for the material samplers.
pub const DEFAULT_ANISOTROPY: u16 = 16;

/// GPU-side material texture storage and its samplers, plus layer bookkeeping.
pub struct MaterialTextureArrays {
    /// Square edge length of every array layer in pixels.
    pub layer_size: u32,
    /// Maximum number of layers each array texture holds.
    pub max_layers: u32,
    /// Mip level count of the array textures.
    pub mip_level_count: u32,
    /// `D2Array` texture holding sRGB (color) layers.
    pub srgb_texture: wgpu::Texture,
    /// `D2Array` texture holding linear (non-color) layers.
    pub linear_texture: wgpu::Texture,
    /// Array view of `srgb_texture`.
    pub srgb_view: wgpu::TextureView,
    /// Array view of `linear_texture`.
    pub linear_view: wgpu::TextureView,
    /// Sampler with repeat addressing.
    pub sampler_repeat: wgpu::Sampler,
    /// Sampler with mirrored repeat addressing.
    pub sampler_mirror: wgpu::Sampler,
    /// Sampler with clamp-to-edge addressing.
    pub sampler_clamp: wgpu::Sampler,
    /// Anisotropy clamp the current samplers were built with.
    pub anisotropy_clamp: u16,
    /// Next unused sRGB layer when the free list is empty.
    pub srgb_next_layer: u32,
    /// Next unused linear layer when the free list is empty.
    pub linear_next_layer: u32,
    /// Map from texture name to its assigned layer and sampling.
    pub layer_map: HashMap<String, MaterialTextureLayer>,
    /// Freed sRGB layers available for reuse.
    pub srgb_free_layers: Vec<u32>,
    /// Freed linear layers available for reuse.
    pub linear_free_layers: Vec<u32>,
    placeholder_pixels: Option<Vec<u8>>,
    bindless: bool,
    bindless_max: u32,
    bindless_textures: Vec<Option<wgpu::Texture>>,
    bindless_views: Vec<wgpu::TextureView>,
    bindless_next: u32,
    bindless_free: Vec<u32>,
    bindless_placeholder_view: Option<wgpu::TextureView>,
    bindless_dirty: bool,
}

fn build_material_sampler(
    device: &wgpu::Device,
    anisotropy_clamp: u16,
    address_mode: wgpu::AddressMode,
    label: &str,
) -> wgpu::Sampler {
    let clamped = anisotropy_clamp.clamp(1, 16);
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        address_mode_w: address_mode,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        anisotropy_clamp: clamped,
        ..Default::default()
    })
}

fn build_wrap_samplers(device: &wgpu::Device, anisotropy_clamp: u16) -> [wgpu::Sampler; 3] {
    [
        build_material_sampler(
            device,
            anisotropy_clamp,
            wgpu::AddressMode::Repeat,
            "Material Texture Array Sampler (Repeat)",
        ),
        build_material_sampler(
            device,
            anisotropy_clamp,
            wgpu::AddressMode::MirrorRepeat,
            "Material Texture Array Sampler (Mirror)",
        ),
        build_material_sampler(
            device,
            anisotropy_clamp,
            wgpu::AddressMode::ClampToEdge,
            "Material Texture Array Sampler (Clamp)",
        ),
    ]
}

impl MaterialTextureArrays {
    /// Creates arrays at the default layer size and layer count.
    pub fn new(device: &wgpu::Device) -> Self {
        Self::with_size(device, DEFAULT_LAYER_SIZE, DEFAULT_MAX_LAYERS)
    }

    /// Creates arrays sized to `layer_size` square with `max_layers` layers.
    pub fn with_size(device: &wgpu::Device, layer_size: u32, max_layers: u32) -> Self {
        let largest = layer_size.max(1);
        let mip_level_count = (largest as f32).log2().floor() as u32 + 1;

        let extent = wgpu::Extent3d {
            width: layer_size,
            height: layer_size,
            depth_or_array_layers: max_layers,
        };

        let srgb_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Material Texture Array (sRGB)"),
            size: extent,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let linear_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Material Texture Array (Linear)"),
            size: extent,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let srgb_view = srgb_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let linear_view = linear_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let anisotropy_clamp = DEFAULT_ANISOTROPY;
        let [sampler_repeat, sampler_mirror, sampler_clamp] =
            build_wrap_samplers(device, anisotropy_clamp);

        Self {
            layer_size,
            max_layers,
            mip_level_count,
            srgb_texture,
            linear_texture,
            srgb_view,
            linear_view,
            sampler_repeat,
            sampler_mirror,
            sampler_clamp,
            anisotropy_clamp,
            srgb_next_layer: 0,
            linear_next_layer: 0,
            layer_map: HashMap::new(),
            srgb_free_layers: Vec::new(),
            linear_free_layers: Vec::new(),
            placeholder_pixels: None,
            bindless: false,
            bindless_max: 0,
            bindless_textures: Vec::new(),
            bindless_views: Vec::new(),
            bindless_next: 0,
            bindless_free: Vec::new(),
            bindless_placeholder_view: None,
            bindless_dirty: false,
        }
    }

    /// Creates arrays in bindless mode, backed by up to `max_textures`
    /// individually sized textures and a white 1x1 placeholder view.
    pub fn new_bindless(device: &wgpu::Device, queue: &wgpu::Queue, max_textures: u32) -> Self {
        let mut arrays = Self::with_size(device, DEFAULT_LAYER_SIZE, 1);
        let placeholder = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Material Bindless Placeholder"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &placeholder,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255u8, 255u8, 255u8, 255u8],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let placeholder_view = placeholder.create_view(&wgpu::TextureViewDescriptor::default());
        arrays.bindless = true;
        arrays.bindless_max = max_textures;
        arrays.bindless_placeholder_view = Some(placeholder_view);
        arrays.bindless_dirty = true;
        arrays
    }

    /// Returns a layer slot to the free list so the next upload can reuse
    /// it. Called after the texture cache evicts a no-longer-referenced
    /// texture so loading additional models can scavenge its array slot.
    pub fn release(&mut self, name: &str) -> Option<MaterialTextureLayer> {
        let layer_info = self.layer_map.remove(name)?;
        if self.bindless {
            let slot = layer_info.layer as usize;
            if slot < self.bindless_textures.len() {
                self.bindless_textures[slot] = None;
                if let Some(placeholder) = self.bindless_placeholder_view.clone() {
                    self.bindless_views[slot] = placeholder;
                }
            }
            self.bindless_free.push(layer_info.layer);
            self.bindless_dirty = true;
            return Some(layer_info);
        }
        match layer_info.usage {
            TextureUsage::Color => self.srgb_free_layers.push(layer_info.layer),
            TextureUsage::Linear => self.linear_free_layers.push(layer_info.layer),
        }
        Some(layer_info)
    }

    fn allocate_bindless_slot(&mut self) -> Option<u32> {
        self.bindless_dirty = true;
        if let Some(reused) = self.bindless_free.pop() {
            return Some(reused);
        }
        if self.bindless_next >= self.bindless_max {
            tracing::error!("Material bindless texture array exhausted");
            return None;
        }
        let index = self.bindless_next;
        self.bindless_next += 1;
        let placeholder = self.bindless_placeholder_view.clone()?;
        self.bindless_textures.push(None);
        self.bindless_views.push(placeholder);
        Some(index)
    }

    /// Allocates a free array layer for `usage`, reusing an evicted slot when
    /// one is available. Returns `None` only when the array is exhausted.
    fn allocate_layer(&mut self, usage: TextureUsage) -> Option<u32> {
        match usage {
            TextureUsage::Color => {
                if let Some(reused) = self.srgb_free_layers.pop() {
                    Some(reused)
                } else if self.srgb_next_layer >= self.max_layers {
                    tracing::error!("Material sRGB texture array exhausted");
                    None
                } else {
                    let layer = self.srgb_next_layer;
                    self.srgb_next_layer += 1;
                    Some(layer)
                }
            }
            TextureUsage::Linear => {
                if let Some(reused) = self.linear_free_layers.pop() {
                    Some(reused)
                } else if self.linear_next_layer >= self.max_layers {
                    tracing::error!("Material linear texture array exhausted");
                    None
                } else {
                    let layer = self.linear_next_layer;
                    self.linear_next_layer += 1;
                    Some(layer)
                }
            }
        }
    }

    /// Claims a stable array layer for `name` and fills it with a neutral white
    /// placeholder so geometry using this texture renders cleanly at its base
    /// color factor until the decoded pixels stream in. The layer index never
    /// changes afterward, so the later [`Self::upload`] writes in place without
    /// any material rewrite. Returns the existing layer when already reserved.
    pub fn reserve_layer(
        &mut self,
        queue: &wgpu::Queue,
        name: String,
        usage: TextureUsage,
        wrap_u: SamplerWrap,
        wrap_v: SamplerWrap,
    ) -> Option<u32> {
        if let Some(existing) = self.layer_map.get(&name) {
            return Some(existing.layer);
        }
        let layer = if self.bindless {
            self.allocate_bindless_slot()?
        } else {
            let layer = self.allocate_layer(usage)?;
            self.write_placeholder(queue, usage, layer);
            layer
        };
        self.layer_map.insert(
            name,
            MaterialTextureLayer {
                usage,
                layer,
                wrap_u,
                wrap_v,
            },
        );
        Some(layer)
    }

    fn write_placeholder(&mut self, queue: &wgpu::Queue, usage: TextureUsage, layer: u32) {
        let pixel_count = (self.layer_size as usize) * (self.layer_size as usize) * 4;
        let placeholder = self
            .placeholder_pixels
            .get_or_insert_with(|| vec![255u8; pixel_count]);
        let texture = match usage {
            TextureUsage::Color => &self.srgb_texture,
            TextureUsage::Linear => &self.linear_texture,
        };
        let mut width = self.layer_size;
        let mut height = self.layer_size;
        for mip_level in 0..self.mip_level_count {
            let bytes = (width as usize) * (height as usize) * 4;
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &placeholder[..bytes],
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
            width = (width / 2).max(1);
            height = (height / 2).max(1);
        }
    }

    /// Writes `request`'s pixels into its layer, resampling to the layer size
    /// and regenerating mips. Reuses the reserved layer when one exists,
    /// otherwise allocates. Returns the layer index, or `None` when exhausted.
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mip_generator: &super::mip_generator::MipGenerator,
        request: MaterialTextureUpload<'_>,
    ) -> Option<u32> {
        if self.bindless {
            return self.upload_bindless(device, queue, mip_generator, request);
        }

        let MaterialTextureUpload {
            name,
            rgba_data,
            width,
            height,
            usage,
            wrap_u,
            wrap_v,
        } = request;

        // Reuse the layer reserved for this name (writing over its placeholder) so
        // the index stays stable; allocate one only for textures uploaded without a
        // prior reservation. The map entry is refreshed at the end either way.
        let layer = match self.layer_map.get(&name) {
            Some(existing) => existing.layer,
            None => self.allocate_layer(usage)?,
        };

        let resampled = resample_to_size(rgba_data, width, height, self.layer_size);
        let texture = match usage {
            TextureUsage::Color => &self.srgb_texture,
            TextureUsage::Linear => &self.linear_texture,
        };

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &resampled,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.layer_size),
                rows_per_image: Some(self.layer_size),
            },
            wgpu::Extent3d {
                width: self.layer_size,
                height: self.layer_size,
                depth_or_array_layers: 1,
            },
        );

        if self.mip_level_count > 1 {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Material Texture Array Mip Gen"),
            });
            mip_generator.generate_mips(device, &mut encoder, texture, layer);
            queue.submit(std::iter::once(encoder.finish()));
        }

        self.layer_map.insert(
            name,
            MaterialTextureLayer {
                usage,
                layer,
                wrap_u,
                wrap_v,
            },
        );
        Some(layer)
    }

    fn upload_bindless(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mip_generator: &super::mip_generator::MipGenerator,
        request: MaterialTextureUpload<'_>,
    ) -> Option<u32> {
        let MaterialTextureUpload {
            name,
            rgba_data,
            width,
            height,
            usage,
            wrap_u,
            wrap_v,
        } = request;
        let slot = match self.layer_map.get(&name) {
            Some(existing) => existing.layer,
            None => self.allocate_bindless_slot()?,
        };

        let largest = width.max(height).max(1);
        let mip_level_count = (largest as f32).log2().floor() as u32 + 1;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Material Bindless Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: usage.wgpu_format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
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
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        if mip_level_count > 1 {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Material Bindless Mip Gen"),
            });
            mip_generator.generate_mips(device, &mut encoder, &texture, 0);
            queue.submit(std::iter::once(encoder.finish()));
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let slot_index = slot as usize;
        self.bindless_views[slot_index] = view;
        self.bindless_textures[slot_index] = Some(texture);
        self.bindless_dirty = true;

        self.layer_map.insert(
            name,
            MaterialTextureLayer {
                usage,
                layer: slot,
                wrap_u,
                wrap_v,
            },
        );
        Some(slot)
    }

    /// Whether the arrays run in bindless mode.
    pub fn is_bindless(&self) -> bool {
        self.bindless
    }

    /// Maximum number of textures the bindless array holds.
    pub fn bindless_max(&self) -> u32 {
        self.bindless_max
    }

    /// Returns and clears the flag marking the bindless views as changed since
    /// the last read.
    pub fn take_bindless_dirty(&mut self) -> bool {
        std::mem::take(&mut self.bindless_dirty)
    }

    /// Bindless texture views in slot order, or the placeholder view alone when
    /// no slots exist yet.
    pub fn bindless_view_refs(&self) -> Vec<&wgpu::TextureView> {
        if self.bindless_views.is_empty() {
            return self.bindless_placeholder_view.iter().collect();
        }
        self.bindless_views.iter().collect()
    }

    /// Looks up the layer registered for `name`.
    pub fn get_layer(&self, name: &str) -> Option<MaterialTextureLayer> {
        self.layer_map.get(name).copied()
    }

    /// Array view of the sRGB texture.
    pub fn srgb_view(&self) -> &wgpu::TextureView {
        &self.srgb_view
    }

    /// Array view of the linear texture.
    pub fn linear_view(&self) -> &wgpu::TextureView {
        &self.linear_view
    }

    /// The repeat, mirror, and clamp samplers, in that order.
    pub fn samplers(&self) -> [wgpu::Sampler; 3] {
        [
            self.sampler_repeat.clone(),
            self.sampler_mirror.clone(),
            self.sampler_clamp.clone(),
        ]
    }

    /// Replaces the wrap samplers with ones using the requested anisotropy
    /// clamp. Returns true when the clamp changed and the samplers were
    /// rebuilt. Callers must update any bind groups that reference the old
    /// samplers.
    pub fn set_anisotropy(&mut self, device: &wgpu::Device, anisotropy_clamp: u16) -> bool {
        let clamped = anisotropy_clamp.clamp(1, 16);
        if clamped == self.anisotropy_clamp {
            return false;
        }
        let [repeat, mirror, clamp] = build_wrap_samplers(device, clamped);
        self.sampler_repeat = repeat;
        self.sampler_mirror = mirror;
        self.sampler_clamp = clamp;
        self.anisotropy_clamp = clamped;
        true
    }
}

fn resample_to_size(rgba_data: &[u8], width: u32, height: u32, target: u32) -> Vec<u8> {
    if width == target && height == target {
        return rgba_data.to_vec();
    }
    let mut out = vec![0u8; (target * target * 4) as usize];
    for y in 0..target {
        let src_y0 = ((y as u64) * (height as u64) / (target as u64)) as u32;
        let src_y1 = ((((y as u64) + 1) * (height as u64) / (target as u64)) as u32)
            .max(src_y0 + 1)
            .min(height);
        for x in 0..target {
            let src_x0 = ((x as u64) * (width as u64) / (target as u64)) as u32;
            let src_x1 = ((((x as u64) + 1) * (width as u64) / (target as u64)) as u32)
                .max(src_x0 + 1)
                .min(width);
            let mut accum = [0u32; 4];
            let mut count = 0u32;
            for sy in src_y0..src_y1 {
                for sx in src_x0..src_x1 {
                    let src_offset = ((sy * width + sx) * 4) as usize;
                    accum[0] += rgba_data[src_offset] as u32;
                    accum[1] += rgba_data[src_offset + 1] as u32;
                    accum[2] += rgba_data[src_offset + 2] as u32;
                    accum[3] += rgba_data[src_offset + 3] as u32;
                    count += 1;
                }
            }
            let divisor = count.max(1);
            let dst_offset = ((y * target + x) * 4) as usize;
            out[dst_offset] = (accum[0] / divisor) as u8;
            out[dst_offset + 1] = (accum[1] / divisor) as u8;
            out[dst_offset + 2] = (accum[2] / divisor) as u8;
            out[dst_offset + 3] = (accum[3] / divisor) as u8;
        }
    }
    out
}
