//! Loads an equirectangular HDR image into a sky cubemap and its filtered
//! irradiance and prefiltered environment maps.

use super::envmap_filter::{FilteredEnvironmentMaps, filter_environment_map};

const CUBEMAP_SIZE: u32 = 1024;

fn calculate_mip_count(size: u32) -> u32 {
    (size as f32).log2().floor() as u32 + 1
}

/// Cubemap produced from an HDR image along with its filtered IBL maps.
pub struct HdriResult {
    /// Mipmapped sky cubemap texture.
    pub cubemap: wgpu::Texture,
    /// Cube-dimension view of `cubemap`.
    pub cubemap_view: wgpu::TextureView,
    /// Irradiance and prefiltered environment maps derived from the cubemap.
    pub filtered_maps: FilteredEnvironmentMaps,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MipParams {
    dst_size: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
}

/// Decodes an equirectangular `.hdr` byte buffer, projects it to a cubemap,
/// generates its mip chain, and filters it into IBL maps.
pub fn load_hdri_texture(device: &wgpu::Device, queue: &wgpu::Queue, bytes: &[u8]) -> HdriResult {
    let decoder = image::codecs::hdr::HdrDecoder::new(bytes).expect("Failed to create HDR decoder");
    let metadata = decoder.metadata();
    let img = image::DynamicImage::from_decoder(decoder).expect("Failed to decode HDR image");
    let rgb_img = img.to_rgb32f();
    let (width, height) = (metadata.width, metadata.height);

    let data: Vec<u16> = rgb_img
        .pixels()
        .flat_map(|pixel| {
            [
                half::f16::from_f32(pixel.0[0]).to_bits(),
                half::f16::from_f32(pixel.0[1]).to_bits(),
                half::f16::from_f32(pixel.0[2]).to_bits(),
                half::f16::from_f32(1.0).to_bits(),
            ]
        })
        .collect();

    let equirect_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Equirectangular Source Texture"),
        size: wgpu::Extent3d {
            width,
            height,
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
            texture: &equirect_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(&data),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4 * 2),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    let mip_level_count = calculate_mip_count(CUBEMAP_SIZE);
    #[cfg(not(target_arch = "wasm32"))]
    let cubemap_usage = if super::envmap_filter::DEBUG_DUMP_CUBEMAPS {
        wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC
    } else {
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING
    };
    #[cfg(target_arch = "wasm32")]
    let cubemap_usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING;

    let cubemap = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Sky Cubemap Texture"),
        size: wgpu::Extent3d {
            width: CUBEMAP_SIZE,
            height: CUBEMAP_SIZE,
            depth_or_array_layers: 6,
        },
        mip_level_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: cubemap_usage,
        view_formats: &[],
    });

    convert_equirect_to_cubemap(device, queue, &equirect_texture, &cubemap);
    generate_cubemap_mipmaps(device, queue, &cubemap);

    let cubemap_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    });

    let filtered_maps = filter_environment_map(device, queue, &cubemap, &cubemap_view);

    HdriResult {
        cubemap,
        cubemap_view,
        filtered_maps,
    }
}

fn convert_equirect_to_cubemap(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    equirect_texture: &wgpu::Texture,
    cubemap: &wgpu::Texture,
) {
    let equirect_view = equirect_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let cubemap_storage_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        base_mip_level: 0,
        mip_level_count: Some(1),
        ..Default::default()
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Equirect to Cube Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Equirect to Cube Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&equirect_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&cubemap_storage_view),
            },
        ],
    });

    let shader = crate::wgpu::shader_compose::compile_wgsl(
        device,
        "equirect_to_cube.wgsl",
        include_str!("shaders/equirect_to_cube.wgsl"),
    );

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Equirect to Cube Pipeline Layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Equirect to Cube Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Equirect to Cube Command Encoder"),
    });

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Equirect to Cube Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(64, 64, 6);
    }

    queue.submit(Some(encoder.finish()));
}

/// Fills mips 1 and above of a cubemap by box-downsampling each face from the
/// previous mip level with a compute pass.
pub fn generate_cubemap_mipmaps(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cubemap: &wgpu::Texture,
) {
    let mip_level_count = cubemap.mip_level_count();
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Cubemap Mipgen Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba16Float,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let shader = crate::wgpu::shader_compose::compile_wgsl(
        device,
        "cubemap_mipgen.wgsl",
        include_str!("shaders/cubemap_mipgen.wgsl"),
    );

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Cubemap Mipgen Pipeline Layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Cubemap Mipgen Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Mipgen Params Buffer"),
        size: std::mem::size_of::<MipParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    for mip_level in 1..mip_level_count {
        let src_mip_level = mip_level - 1;
        let dst_size = CUBEMAP_SIZE >> mip_level;

        let src_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            base_mip_level: src_mip_level,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let dst_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            base_mip_level: mip_level,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cubemap Mipgen Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&dst_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let params = MipParams {
            dst_size,
            _padding0: 0,
            _padding1: 0,
            _padding2: 0,
        };

        queue.write_buffer(&params_buffer, 0, bytemuck::cast_slice(&[params]));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Cubemap Mipgen Command Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Cubemap Mipgen Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(dst_size.div_ceil(16), dst_size.div_ceil(16), 6);
        }

        queue.submit(Some(encoder.finish()));
    }
}

/// Creates a 1x1 placeholder cubemap and its cube view for when no HDRI is set.
pub fn create_dummy_cubemap(device: &wgpu::Device) -> (wgpu::Texture, wgpu::TextureView) {
    let cubemap = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Dummy Cubemap Texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let cubemap_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    });

    (cubemap, cubemap_view)
}
