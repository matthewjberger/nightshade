//! Linearly-transformed-cosine lookup tables used to shade area light specular.
//!
//! The two 64x64 RGBA16F tables are the fitted Heitz LTC data: table one holds
//! the inverse transform matrix, table two the BRDF magnitude and Fresnel terms.

const LTC_MAT_1: &[u8] = include_bytes!("ltc/ltc_mat_1.bin");
const LTC_MAT_2: &[u8] = include_bytes!("ltc/ltc_mat_2.bin");
const LTC_LUT_SIZE: u32 = 64;

pub struct LtcTextures {
    pub view_matrix: wgpu::TextureView,
    pub view_terms: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    _matrix: wgpu::Texture,
    _terms: wgpu::Texture,
}

pub fn create(device: &wgpu::Device, queue: &wgpu::Queue) -> LtcTextures {
    let matrix = create_lut(device, queue, "LTC Matrix LUT", LTC_MAT_1);
    let terms = create_lut(device, queue, "LTC Terms LUT", LTC_MAT_2);
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("LTC Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    });
    LtcTextures {
        view_matrix: matrix.create_view(&wgpu::TextureViewDescriptor::default()),
        view_terms: terms.create_view(&wgpu::TextureViewDescriptor::default()),
        sampler,
        _matrix: matrix,
        _terms: terms,
    }
}

fn create_lut(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    data: &[u8],
) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: LTC_LUT_SIZE,
            height: LTC_LUT_SIZE,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(LTC_LUT_SIZE * 4 * 2),
            rows_per_image: Some(LTC_LUT_SIZE),
        },
        wgpu::Extent3d {
            width: LTC_LUT_SIZE,
            height: LTC_LUT_SIZE,
            depth_or_array_layers: 1,
        },
    );
    texture
}
