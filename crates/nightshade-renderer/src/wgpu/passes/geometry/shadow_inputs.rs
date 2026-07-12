use crate::wgpu::passes::SpotlightShadowData;
use crate::wgpu::passes::geometry::projection::MAX_SPOTLIGHT_SHADOWS;
use crate::wgpu::passes::shadow_depth::{MAX_POINT_LIGHT_SHADOWS, PointLightShadowData};

/// Nearest-filtered clamping sampler used to read shadow maps. Shared by the
/// mesh and skinned mesh passes for directional/spot and point shadow lookups.
pub fn shadow_comparison_sampler(device: &wgpu::Device, label: &str) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

/// Depth-comparison sampler for hardware PCF shadow filtering. Linear filtering
/// gives smooth sub-texel percentage-closer results. Reversed-Z depth means the
/// receiver is lit when its depth is greater or equal to the stored depth.
pub fn shadow_pcf_sampler(device: &wgpu::Device, label: &str) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(label),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        compare: Some(wgpu::CompareFunction::GreaterEqual),
        ..Default::default()
    })
}

/// Storage buffer sized for the spotlight shadow table. Shared by the mesh and
/// skinned mesh passes.
pub fn spotlight_shadow_buffer(device: &wgpu::Device, label: &str) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: (std::mem::size_of::<SpotlightShadowData>() * MAX_SPOTLIGHT_SHADOWS) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// Storage buffer sized for the point-light shadow table. Shared by the mesh
/// and skinned mesh passes.
pub fn point_shadow_buffer(device: &wgpu::Device, label: &str) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: (std::mem::size_of::<PointLightShadowData>() * MAX_POINT_LIGHT_SHADOWS) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// A 1x1 R32Float cube-array view standing in for the point-shadow cubemap
/// before any shadows are rendered. The backing texture is owned by the
/// returned view. Shared by the mesh and skinned mesh passes.
pub fn dummy_point_shadow_cubemap_view(device: &wgpu::Device, label: &str) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 6 * MAX_POINT_LIGHT_SHADOWS as u32,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::CubeArray),
        ..Default::default()
    })
}
