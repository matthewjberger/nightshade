use crate::renderer::wgpu::build;
use crate::renderer::wgpu::shader_compose::ShaderComposer;
const LUT_SIZE: u32 = 256;

pub fn generate_brdf_lut(
    device: &wgpu::Device,
    composer: &ShaderComposer,
    queue: &wgpu::Queue,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("BRDF LUT Texture"),
        size: crate::renderer::texture_extent::texture_extent(device, LUT_SIZE, LUT_SIZE, 1),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bind_group_layout = build::bind_layout(
        device,
        "BRDF LUT Bind Group Layout",
        &[build::storage_texture_at(
            0,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::StorageTextureAccess::WriteOnly,
            wgpu::TextureViewDimension::D2,
            wgpu::ShaderStages::COMPUTE,
        )],
    );

    let bind_group = build::binding(
        device,
        "BRDF LUT Bind Group",
        &bind_group_layout,
        vec![wgpu::BindingResource::TextureView(&texture_view)],
    );

    let shader = composer.compile_wgsl(
        device,
        "brdf_lut.wgsl",
        include_str!("shaders/brdf_lut.wgsl"),
    );

    let pipeline_layout =
        build::pipeline_layout(device, "BRDF LUT Pipeline Layout", &[&bind_group_layout]);

    let pipeline = build::compute(
        device,
        "BRDF LUT Compute Pipeline",
        &shader,
        "main",
        &pipeline_layout,
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("BRDF LUT Command Encoder"),
    });

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("BRDF LUT Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(LUT_SIZE / 16, LUT_SIZE / 16, 1);
    }

    queue.submit(Some(encoder.finish()));

    (texture, texture_view)
}
