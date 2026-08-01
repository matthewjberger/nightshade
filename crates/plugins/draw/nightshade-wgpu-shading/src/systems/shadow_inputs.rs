use crate::systems::light::PointLightShadowData;
use crate::systems::projection::MAX_SPOTLIGHT_SHADOWS;
use nightshade_render::data::shadow::SpotlightShadowData;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::backend::caps::MAX_POINT_LIGHT_SHADOWS;
use nightshade_wgpu::wgpu;

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

pub fn spotlight_shadow_buffer(device: &wgpu::Device, label: &str) -> wgpu::Buffer {
    build::storage(
        device,
        label,
        (std::mem::size_of::<SpotlightShadowData>() * MAX_SPOTLIGHT_SHADOWS) as u64,
    )
}

pub fn point_shadow_buffer(device: &wgpu::Device, label: &str) -> wgpu::Buffer {
    build::storage(
        device,
        label,
        (std::mem::size_of::<PointLightShadowData>() * MAX_POINT_LIGHT_SHADOWS) as u64,
    )
}

pub fn dummy_point_shadow_cubemap_view(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
) -> wgpu::TextureView {
    let texture = nightshade_wgpu::backend::placeholder::placeholder_texture(
        device,
        queue,
        label,
        wgpu::TextureFormat::R32Float,
        nightshade_wgpu::backend::caps::point_shadow_layers(device),
        &nightshade_wgpu::backend::placeholder::FAR_R32,
    );
    texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::CubeArray),
        ..Default::default()
    })
}

pub fn scene_bind_group_layout(device: &wgpu::Device, label: &str) -> wgpu::BindGroupLayout {
    build::bind_layout(
        device,
        label,
        &[
            build::depth_at(0, wgpu::ShaderStages::FRAGMENT),
            build::sampler_at(
                1,
                wgpu::SamplerBindingType::NonFiltering,
                wgpu::ShaderStages::FRAGMENT,
            ),
            build::depth_at(2, wgpu::ShaderStages::FRAGMENT),
            build::sampler_at(
                3,
                wgpu::SamplerBindingType::NonFiltering,
                wgpu::ShaderStages::FRAGMENT,
            ),
            build::storage_at(4, false, wgpu::ShaderStages::FRAGMENT),
            build::painted_at(5),
            build::filtering_at(6),
            build::texture_at(
                7,
                wgpu::TextureSampleType::Float { filterable: true },
                wgpu::TextureViewDimension::Cube,
                wgpu::ShaderStages::FRAGMENT,
            ),
            build::filtering_at(8),
            build::texture_at(
                9,
                wgpu::TextureSampleType::Float { filterable: true },
                wgpu::TextureViewDimension::Cube,
                wgpu::ShaderStages::FRAGMENT,
            ),
            build::filtering_at(10),
            build::texture_at(
                11,
                wgpu::TextureSampleType::Float { filterable: false },
                wgpu::TextureViewDimension::CubeArray,
                wgpu::ShaderStages::FRAGMENT,
            ),
            build::sampler_at(
                12,
                wgpu::SamplerBindingType::NonFiltering,
                wgpu::ShaderStages::FRAGMENT,
            ),
            build::storage_at(13, false, wgpu::ShaderStages::FRAGMENT),
            build::texture_at(
                14,
                wgpu::TextureSampleType::Float { filterable: true },
                wgpu::TextureViewDimension::Cube,
                wgpu::ShaderStages::FRAGMENT,
            ),
            build::texture_at(
                15,
                wgpu::TextureSampleType::Float { filterable: true },
                wgpu::TextureViewDimension::Cube,
                wgpu::ShaderStages::FRAGMENT,
            ),
            build::painted_at(16),
            build::filtering_at(17),
            build::sampler_at(
                18,
                wgpu::SamplerBindingType::Comparison,
                wgpu::ShaderStages::FRAGMENT,
            ),
            build::painted_at(19),
            build::painted_at(20),
            build::filtering_at(21),
            build::texture_at(
                22,
                wgpu::TextureSampleType::Float { filterable: true },
                wgpu::TextureViewDimension::D3,
                wgpu::ShaderStages::FRAGMENT,
            ),
            build::filtering_at(23),
            build::uniform_at(24, wgpu::ShaderStages::FRAGMENT),
        ],
    )
}
