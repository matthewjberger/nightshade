use super::envmap_filter::{FilteredEnvironmentMaps, filter_environment_map};
use nightshade_wgpu::backend::PassBuild;
use nightshade_wgpu::backend::build;
use nightshade_wgpu::wgpu;

const CUBEMAP_SIZE: u32 = 1024;

fn calculate_mip_count(size: u32) -> u32 {
    (size as f32).log2().floor() as u32 + 1
}

pub struct HdriResult {
    pub cubemap: wgpu::Texture,
    pub cubemap_view: wgpu::TextureView,
    pub filtered_maps: FilteredEnvironmentMaps,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MipParams {
    dst_size: u32,
    padding_0: u32,
    padding_1: u32,
    padding_2: u32,
}

pub fn load_hdri_texture(build: PassBuild, queue: &wgpu::Queue, bytes: &[u8]) -> HdriResult {
    let PassBuild { device, .. } = build;
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

    let equirect_texture = build::surface(
        device,
        "Equirectangular Source Texture",
        wgpu::TextureFormat::Rgba16Float,
        [width, height, 1],
        1,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );

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
        size: nightshade_wgpu::backend::texture_extent(device, CUBEMAP_SIZE, CUBEMAP_SIZE, 6),
        mip_level_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: cubemap_usage,
        view_formats: &[],
    });

    convert_equirect_to_cubemap(build, queue, &equirect_texture, &cubemap);
    generate_cubemap_mipmaps(build, queue, &cubemap);

    let cubemap_view = cubemap.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    });

    let filtered_maps = filter_environment_map(build, queue, &cubemap, &cubemap_view);

    HdriResult {
        cubemap,
        cubemap_view,
        filtered_maps,
    }
}

fn convert_equirect_to_cubemap(
    build: PassBuild,
    queue: &wgpu::Queue,
    equirect_texture: &wgpu::Texture,
    cubemap: &wgpu::Texture,
) {
    let PassBuild {
        device, composer, ..
    } = build;
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

    let bind_group_layout = build::bind_layout(
        device,
        "Equirect to Cube Bind Group Layout",
        &[
            build::texture_at(
                0,
                wgpu::TextureSampleType::Float { filterable: true },
                wgpu::TextureViewDimension::D2,
                wgpu::ShaderStages::COMPUTE,
            ),
            build::sampler_at(
                1,
                wgpu::SamplerBindingType::Filtering,
                wgpu::ShaderStages::COMPUTE,
            ),
            build::storage_texture_at(
                2,
                wgpu::TextureFormat::Rgba16Float,
                wgpu::StorageTextureAccess::WriteOnly,
                wgpu::TextureViewDimension::D2Array,
                wgpu::ShaderStages::COMPUTE,
            ),
        ],
    );

    let bind_group = build::binding(
        device,
        "Equirect to Cube Bind Group",
        &bind_group_layout,
        vec![
            wgpu::BindingResource::TextureView(&equirect_view),
            wgpu::BindingResource::Sampler(&sampler),
            wgpu::BindingResource::TextureView(&cubemap_storage_view),
        ],
    );

    let shader = composer.compile_wgsl(
        device,
        "equirect_to_cube.wgsl",
        include_str!("../shaders/equirect_to_cube.wgsl"),
    );

    let pipeline_layout = build::pipeline_layout(
        device,
        "Equirect to Cube Pipeline Layout",
        &[&bind_group_layout],
    );

    let pipeline = build::compute(
        device,
        "Equirect to Cube Pipeline",
        &shader,
        "main",
        &pipeline_layout,
    );

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

pub fn generate_cubemap_mipmaps(build: PassBuild, queue: &wgpu::Queue, cubemap: &wgpu::Texture) {
    let PassBuild {
        device, composer, ..
    } = build;
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

    let bind_group_layout = build::bind_layout(
        device,
        "Cubemap Mipgen Bind Group Layout",
        &[
            build::texture_at(
                0,
                wgpu::TextureSampleType::Float { filterable: true },
                wgpu::TextureViewDimension::D2Array,
                wgpu::ShaderStages::COMPUTE,
            ),
            build::sampler_at(
                1,
                wgpu::SamplerBindingType::Filtering,
                wgpu::ShaderStages::COMPUTE,
            ),
            build::storage_texture_at(
                2,
                wgpu::TextureFormat::Rgba16Float,
                wgpu::StorageTextureAccess::WriteOnly,
                wgpu::TextureViewDimension::D2Array,
                wgpu::ShaderStages::COMPUTE,
            ),
            build::uniform_at(3, wgpu::ShaderStages::COMPUTE),
        ],
    );

    let shader = composer.compile_wgsl(
        device,
        "cubemap_mipgen.wgsl",
        include_str!("../shaders/cubemap_mipgen.wgsl"),
    );

    let pipeline_layout = build::pipeline_layout(
        device,
        "Cubemap Mipgen Pipeline Layout",
        &[&bind_group_layout],
    );

    let pipeline = build::compute(
        device,
        "Cubemap Mipgen Compute Pipeline",
        &shader,
        "main",
        &pipeline_layout,
    );

    let params_buffer = build::uniform(
        device,
        "Mipgen Params Buffer",
        std::mem::size_of::<MipParams>() as u64,
    );

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

        let bind_group = build::binding(
            device,
            "Cubemap Mipgen Bind Group",
            &bind_group_layout,
            vec![
                wgpu::BindingResource::TextureView(&src_view),
                wgpu::BindingResource::Sampler(&sampler),
                wgpu::BindingResource::TextureView(&dst_view),
                params_buffer.as_entire_binding(),
            ],
        );

        let params = MipParams {
            dst_size,
            padding_0: 0,
            padding_1: 0,
            padding_2: 0,
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
