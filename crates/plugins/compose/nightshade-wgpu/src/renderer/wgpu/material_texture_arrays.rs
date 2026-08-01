use crate::renderer::wgpu::build;
use nightshade_render::data::texture_data::{SamplerFilter, SamplerWrap, TextureUsage};
use nightshade_render::queries::image::resample_to_size;
use nightshade_render::queries::texture::pack_texture_slot;
use std::collections::HashMap;

const DEFAULT_LAYER_SIZE: u32 = 1024;
const DEFAULT_MAX_LAYERS: u32 = 256;

pub const MATERIAL_SAMPLER_COUNT: usize = 6;

#[derive(Copy, Clone, Debug)]
pub struct MaterialTextureLayer {
    pub usage: TextureUsage,
    pub layer: u32,
    pub wrap_u: SamplerWrap,
    pub wrap_v: SamplerWrap,
    pub filter: SamplerFilter,
}

impl MaterialTextureLayer {
    pub fn packed(&self) -> u32 {
        pack_texture_slot(self.layer, self.wrap_u, self.wrap_v, self.filter)
    }
}

pub struct MaterialTextureUpload<'a> {
    pub name: String,
    pub rgba_data: &'a [u8],
    pub width: u32,
    pub height: u32,
    pub usage: TextureUsage,
    pub wrap_u: SamplerWrap,
    pub wrap_v: SamplerWrap,
    pub filter: SamplerFilter,
}

pub const DEFAULT_ANISOTROPY: u16 = 16;

pub struct MaterialTextureArrays {
    pub layer_size: u32,
    pub max_layers: u32,
    pub mip_level_count: u32,
    pub srgb_texture: wgpu::Texture,
    pub linear_texture: wgpu::Texture,
    pub srgb_view: wgpu::TextureView,
    pub linear_view: wgpu::TextureView,
    samplers: [wgpu::Sampler; MATERIAL_SAMPLER_COUNT],
    pub anisotropy_clamp: u16,
    pub srgb_next_layer: u32,
    pub linear_next_layer: u32,
    pub layer_map: HashMap<String, MaterialTextureLayer>,
    pub srgb_free_layers: Vec<u32>,
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
    filter: SamplerFilter,
    label: &str,
) -> wgpu::Sampler {
    let (filter_mode, clamped) = match filter {
        SamplerFilter::Linear => (wgpu::FilterMode::Linear, anisotropy_clamp.clamp(1, 16)),
        SamplerFilter::Nearest => (wgpu::FilterMode::Nearest, 1),
    };
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        address_mode_w: address_mode,
        mag_filter: filter_mode,
        min_filter: filter_mode,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        anisotropy_clamp: clamped,
        ..Default::default()
    })
}

fn build_wrap_samplers(
    device: &wgpu::Device,
    anisotropy_clamp: u16,
) -> [wgpu::Sampler; MATERIAL_SAMPLER_COUNT] {
    [
        build_material_sampler(
            device,
            anisotropy_clamp,
            wgpu::AddressMode::Repeat,
            SamplerFilter::Linear,
            "Material Texture Array Sampler (Repeat)",
        ),
        build_material_sampler(
            device,
            anisotropy_clamp,
            wgpu::AddressMode::MirrorRepeat,
            SamplerFilter::Linear,
            "Material Texture Array Sampler (Mirror)",
        ),
        build_material_sampler(
            device,
            anisotropy_clamp,
            wgpu::AddressMode::ClampToEdge,
            SamplerFilter::Linear,
            "Material Texture Array Sampler (Clamp)",
        ),
        build_material_sampler(
            device,
            anisotropy_clamp,
            wgpu::AddressMode::Repeat,
            SamplerFilter::Nearest,
            "Material Texture Array Sampler (Repeat, Nearest)",
        ),
        build_material_sampler(
            device,
            anisotropy_clamp,
            wgpu::AddressMode::MirrorRepeat,
            SamplerFilter::Nearest,
            "Material Texture Array Sampler (Mirror, Nearest)",
        ),
        build_material_sampler(
            device,
            anisotropy_clamp,
            wgpu::AddressMode::ClampToEdge,
            SamplerFilter::Nearest,
            "Material Texture Array Sampler (Clamp, Nearest)",
        ),
    ]
}

impl MaterialTextureArrays {
    pub fn new(device: &wgpu::Device) -> Self {
        Self::with_size(device, DEFAULT_LAYER_SIZE, DEFAULT_MAX_LAYERS)
    }

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
        let samplers = build_wrap_samplers(device, anisotropy_clamp);

        Self {
            layer_size,
            max_layers,
            mip_level_count,
            srgb_texture,
            linear_texture,
            srgb_view,
            linear_view,
            samplers,
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

    pub fn new_bindless(device: &wgpu::Device, queue: &wgpu::Queue, max_textures: u32) -> Self {
        let mut arrays = Self::with_size(device, DEFAULT_LAYER_SIZE, 1);
        let placeholder = build::surface(
            device,
            "Material Bindless Placeholder",
            wgpu::TextureFormat::Rgba8Unorm,
            [1, 1, 1],
            1,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
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

    pub fn reserve_layer(
        &mut self,
        queue: &wgpu::Queue,
        name: String,
        usage: TextureUsage,
        wrap_u: SamplerWrap,
        wrap_v: SamplerWrap,
        filter: SamplerFilter,
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
                filter,
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
            filter,
        } = request;

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
                filter,
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
            filter,
        } = request;
        let slot = match self.layer_map.get(&name) {
            Some(existing) => existing.layer,
            None => self.allocate_bindless_slot()?,
        };

        let largest = width.max(height).max(1);
        let mip_level_count = (largest as f32).log2().floor() as u32 + 1;
        let texture = build::surface(
            device,
            "Material Bindless Texture",
            crate::renderer::wgpu::texture_cache::texture_usage_format(usage),
            [width, height, 1],
            mip_level_count,
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
        );

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
                filter,
            },
        );
        Some(slot)
    }

    pub fn is_bindless(&self) -> bool {
        self.bindless
    }

    pub fn bindless_max(&self) -> u32 {
        self.bindless_max
    }

    pub fn take_bindless_dirty(&mut self) -> bool {
        std::mem::take(&mut self.bindless_dirty)
    }

    pub fn bindless_view_refs(&self) -> Vec<&wgpu::TextureView> {
        if self.bindless_views.is_empty() {
            return self.bindless_placeholder_view.iter().collect();
        }
        self.bindless_views.iter().collect()
    }

    pub fn get_layer(&self, name: &str) -> Option<MaterialTextureLayer> {
        self.layer_map.get(name).copied()
    }

    pub fn srgb_view(&self) -> &wgpu::TextureView {
        &self.srgb_view
    }

    pub fn linear_view(&self) -> &wgpu::TextureView {
        &self.linear_view
    }

    pub fn samplers(&self) -> [wgpu::Sampler; MATERIAL_SAMPLER_COUNT] {
        self.samplers.clone()
    }

    pub fn set_anisotropy(&mut self, device: &wgpu::Device, anisotropy_clamp: u16) -> bool {
        let clamped = anisotropy_clamp.clamp(1, 16);
        if clamped == self.anisotropy_clamp {
            return false;
        }
        self.samplers = build_wrap_samplers(device, clamped);
        self.anisotropy_clamp = clamped;
        true
    }
}
