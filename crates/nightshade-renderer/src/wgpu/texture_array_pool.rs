//! A D2-array texture pool: a fixed set of same-size RGBA8 layers sampled by
//! layer index. Adding an image is one [`upload_layer`] into a free layer, so
//! it is O(1) up to `max_layers`, and every layer shares a single view and
//! sampler. Layer assignment is the caller's; the pool is just the GPU resource.
//!
//! Suitable for pools of small, same-size images — UI thumbnails, icons, and
//! asset previews are the engine's use. WebGPU guarantees at least 256 array
//! layers and `D2Array` views are core, so it works on every backend the engine
//! targets, including wasm.

/// A pool of same-size RGBA8 texture-array layers behind one shared view and
/// sampler.
pub struct TextureArrayPool {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub layer_size: u32,
    pub max_layers: u32,
}

/// Creates a pool of `max_layers` square `layer_size` RGBA8 layers with clamped
/// linear sampling.
pub fn create_texture_array_pool(
    device: &wgpu::Device,
    layer_size: u32,
    max_layers: u32,
) -> TextureArrayPool {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Texture Array Pool"),
        size: wgpu::Extent3d {
            width: layer_size,
            height: layer_size,
            depth_or_array_layers: max_layers,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("Texture Array Pool View"),
        format: Some(wgpu::TextureFormat::Rgba8Unorm),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        usage: None,
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(max_layers),
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Texture Array Pool Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });

    TextureArrayPool {
        texture,
        view,
        sampler,
        layer_size,
        max_layers,
    }
}

/// Writes `rgba` into `layer`. A no-op if `layer` is out of range or the image
/// exceeds the layer size.
pub fn upload_layer(
    pool: &TextureArrayPool,
    queue: &wgpu::Queue,
    layer: u32,
    rgba: &[u8],
    width: u32,
    height: u32,
) {
    if layer >= pool.max_layers || width > pool.layer_size || height > pool.layer_size {
        return;
    }
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &pool.texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: 0,
                y: 0,
                z: layer,
            },
            aspect: wgpu::TextureAspect::All,
        },
        rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}
