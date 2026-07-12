const IRRADIANCE_SIZE: u32 = 64;
const PREFILTERED_SIZE: u32 = 512;
const PREFILTERED_MIP_LEVELS: u32 = 5;
const IRRADIANCE_SAMPLES: u32 = 1024;
const PREFILTERED_SAMPLES: u32 = 512;

const DISTRIBUTION_LAMBERTIAN: u32 = 0;
const DISTRIBUTION_GGX: u32 = 1;

pub const DEBUG_DUMP_CUBEMAPS: bool = false;

#[cfg(not(target_arch = "wasm32"))]
fn dump_cubemap_to_disk(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    name: &str,
    mip_level: u32,
) {
    let size = texture.size().width >> mip_level;
    let bytes_per_pixel = 8u32;
    let unpadded_bytes_per_row = size * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
    let buffer_size = (padded_bytes_per_row * size * 6) as u64;

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Cubemap Debug Staging Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Cubemap Debug Copy Encoder"),
    });

    for face in 0..6u32 {
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: face,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: (padded_bytes_per_row * size * face) as u64,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(size),
                },
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
        );
    }

    queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });
    while receiver.try_recv().is_err() {
        let _ = device.poll(wgpu::PollType::Poll);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    let data = buffer_slice.get_mapped_range();

    let face_names = ["+X", "-X", "+Y", "-Y", "+Z", "-Z"];

    let output_dir = std::path::Path::new("debug_cubemaps");
    std::fs::create_dir_all(output_dir).ok();

    for face in 0..6u32 {
        let face_offset = (padded_bytes_per_row * size * face) as usize;
        let mut pixels = Vec::with_capacity((size * size * 4) as usize);

        for row in 0..size {
            let row_start = face_offset + (row * padded_bytes_per_row) as usize;
            for col in 0..size {
                let pixel_offset = row_start + (col * bytes_per_pixel) as usize;
                let r =
                    half::f16::from_le_bytes([data[pixel_offset], data[pixel_offset + 1]]).to_f32();
                let g = half::f16::from_le_bytes([data[pixel_offset + 2], data[pixel_offset + 3]])
                    .to_f32();
                let b = half::f16::from_le_bytes([data[pixel_offset + 4], data[pixel_offset + 5]])
                    .to_f32();

                let tone_map = |x: f32| -> u8 {
                    let mapped = x / (1.0 + x);
                    (mapped.powf(1.0 / 2.2).clamp(0.0, 1.0) * 255.0) as u8
                };

                pixels.push(tone_map(r));
                pixels.push(tone_map(g));
                pixels.push(tone_map(b));
                pixels.push(255u8);
            }
        }

        let filename = format!(
            "{}_mip{}_{}.png",
            name,
            mip_level,
            face_names[face as usize]
                .replace("+", "pos")
                .replace("-", "neg")
        );
        let path = output_dir.join(&filename);

        if let Some(img) = image::RgbaImage::from_raw(size, size, pixels) {
            if let Err(error) = img.save(&path) {
                eprintln!("Failed to save {}: {}", path.display(), error);
            } else {
                println!("Saved: {}", path.display());
            }
        }
    }

    drop(data);
    staging_buffer.unmap();
}

#[cfg(not(target_arch = "wasm32"))]
fn dump_source_cubemap_to_disk(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) {
    for mip in 0..texture.mip_level_count().min(5) {
        dump_cubemap_to_disk(device, queue, texture, "source", mip);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FilterParams {
    face: u32,
    output_size: u32,
    roughness: f32,
    sample_count: u32,
    source_width: u32,
    source_height: u32,
    distribution: u32,
    source_mip_count: u32,
}

pub struct FilteredEnvironmentMaps {
    pub irradiance_texture: wgpu::Texture,
    pub irradiance_view: wgpu::TextureView,
    pub prefiltered_texture: wgpu::Texture,
    pub prefiltered_view: wgpu::TextureView,
}

pub fn filter_environment_map(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source_cubemap: &wgpu::Texture,
    source_cubemap_view: &wgpu::TextureView,
) -> FilteredEnvironmentMaps {
    let source_size = source_cubemap.size();
    let source_mip_count = source_cubemap.mip_level_count();

    #[cfg(not(target_arch = "wasm32"))]
    let output_usage = if DEBUG_DUMP_CUBEMAPS {
        wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::COPY_SRC
    } else {
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING
    };
    #[cfg(target_arch = "wasm32")]
    let output_usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING;

    let irradiance_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Irradiance Cubemap"),
        size: wgpu::Extent3d {
            width: IRRADIANCE_SIZE,
            height: IRRADIANCE_SIZE,
            depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: output_usage,
        view_formats: &[],
    });

    let prefiltered_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Prefiltered Cubemap"),
        size: wgpu::Extent3d {
            width: PREFILTERED_SIZE,
            height: PREFILTERED_SIZE,
            depth_or_array_layers: 6,
        },
        mip_level_count: PREFILTERED_MIP_LEVELS,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: output_usage,
        view_formats: &[],
    });

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
        label: Some("Filter Envmap Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::Cube,
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
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let shader = crate::wgpu::shader_compose::compile_wgsl(
        device,
        "filter_envmap.wgsl",
        include_str!("shaders/filter_envmap.wgsl"),
    );

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Filter Envmap Pipeline Layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Filter Envmap Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let project_sh_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("SH Projection Compute Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("project_sh"),
        compilation_options: Default::default(),
        cache: None,
    });

    let sh_coefficients_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("SH Irradiance Coefficients Buffer"),
        size: 9 * std::mem::size_of::<[f32; 4]>() as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let irradiance_storage_view = irradiance_texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });

    let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Filter Params Buffer"),
        size: std::mem::size_of::<FilterParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let irradiance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Irradiance Filter Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source_cubemap_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&irradiance_storage_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: sh_coefficients_buffer.as_entire_binding(),
            },
        ],
    });

    let irradiance_params = FilterParams {
        face: 0,
        output_size: IRRADIANCE_SIZE,
        roughness: 1.0,
        sample_count: IRRADIANCE_SAMPLES,
        source_width: source_size.width,
        source_height: source_size.height,
        distribution: DISTRIBUTION_LAMBERTIAN,
        source_mip_count,
    };

    queue.write_buffer(
        &params_buffer,
        0,
        bytemuck::cast_slice(&[irradiance_params]),
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Irradiance Filter Command Encoder"),
    });

    {
        let mut sh_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("SH Projection Pass"),
            timestamp_writes: None,
        });
        sh_pass.set_pipeline(&project_sh_pipeline);
        sh_pass.set_bind_group(0, &irradiance_bind_group, &[]);
        sh_pass.dispatch_workgroups(1, 1, 1);
    }

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Irradiance Filter Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &irradiance_bind_group, &[]);
        compute_pass.dispatch_workgroups(
            IRRADIANCE_SIZE.div_ceil(16),
            IRRADIANCE_SIZE.div_ceil(16),
            6,
        );
    }

    queue.submit(Some(encoder.finish()));

    for mip_level in 0..PREFILTERED_MIP_LEVELS {
        let mip_size = PREFILTERED_SIZE >> mip_level;
        let roughness = mip_level as f32 / (PREFILTERED_MIP_LEVELS - 1) as f32;

        let prefiltered_mip_view = prefiltered_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            base_mip_level: mip_level,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let prefiltered_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Prefiltered Filter Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(source_cubemap_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&prefiltered_mip_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: sh_coefficients_buffer.as_entire_binding(),
                },
            ],
        });

        let prefiltered_params = FilterParams {
            face: 0,
            output_size: mip_size,
            roughness,
            sample_count: PREFILTERED_SAMPLES,
            source_width: source_size.width,
            source_height: source_size.height,
            distribution: DISTRIBUTION_GGX,
            source_mip_count,
        };

        queue.write_buffer(
            &params_buffer,
            0,
            bytemuck::cast_slice(&[prefiltered_params]),
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Prefiltered Envmap Command Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Prefiltered Filter Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &prefiltered_bind_group, &[]);
            compute_pass.dispatch_workgroups(mip_size.div_ceil(16), mip_size.div_ceil(16), 6);
        }

        queue.submit(Some(encoder.finish()));
    }

    let irradiance_view = irradiance_texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    });

    let prefiltered_view = prefiltered_texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    });

    #[cfg(not(target_arch = "wasm32"))]
    if DEBUG_DUMP_CUBEMAPS {
        println!("Dumping source cubemap...");
        dump_source_cubemap_to_disk(device, queue, source_cubemap);

        println!("Dumping irradiance cubemap...");
        dump_cubemap_to_disk(device, queue, &irradiance_texture, "irradiance", 0);

        println!("Dumping prefiltered cubemap...");
        for mip in 0..PREFILTERED_MIP_LEVELS {
            dump_cubemap_to_disk(device, queue, &prefiltered_texture, "prefiltered", mip);
        }
        println!("Cubemap dump complete!");
    }

    FilteredEnvironmentMaps {
        irradiance_texture,
        irradiance_view,
        prefiltered_texture,
        prefiltered_view,
    }
}
