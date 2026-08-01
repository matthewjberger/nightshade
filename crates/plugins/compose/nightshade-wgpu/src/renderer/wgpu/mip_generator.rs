use crate::renderer::wgpu::build;
use crate::renderer::wgpu::shader_compose::ShaderComposer;
use std::num::NonZeroU64;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MipParams {
    layer: u32,
    padding_0: u32,
    padding_1: u32,
    padding_2: u32,
}

pub struct MipGenerator {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_srgb: wgpu::RenderPipeline,
    pipeline_linear: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
}

impl MipGenerator {
    pub fn new(device: &wgpu::Device, composer: &ShaderComposer) -> Self {
        let shader = composer.compile_wgsl(
            device,
            "mip_downsample_2d_array.wgsl",
            include_str!("shaders/mip_downsample_2d_array.wgsl"),
        );

        let bind_group_layout = build::bind_layout(
            device,
            "Mip Generator Bind Group Layout",
            &[
                build::texture_at(
                    0,
                    wgpu::TextureSampleType::Float { filterable: true },
                    wgpu::TextureViewDimension::D2Array,
                    wgpu::ShaderStages::FRAGMENT,
                ),
                build::filtering_at(1),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(std::mem::size_of::<MipParams>() as u64),
                    },
                    count: None,
                },
            ],
        );

        let pipeline_layout = build::pipeline_layout(
            device,
            "Mip Generator Pipeline Layout",
            &[&bind_group_layout],
        );

        let make_pipeline = |format: wgpu::TextureFormat, label: &str| {
            build::Pipe::new(device, label, &shader)
                .layout(&pipeline_layout)
                .target(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })
                .vertex("vs_main")
                .fragment("fs_main")
                .make()
        };

        let pipeline_srgb = make_pipeline(
            wgpu::TextureFormat::Rgba8UnormSrgb,
            "Mip Generator Pipeline (sRGB)",
        );
        let pipeline_linear = make_pipeline(
            wgpu::TextureFormat::Rgba8Unorm,
            "Mip Generator Pipeline (Linear)",
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Mip Generator Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Self {
            bind_group_layout,
            pipeline_srgb,
            pipeline_linear,
            sampler,
        }
    }

    pub fn generate_mips(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::Texture,
        layer: u32,
    ) {
        let pipeline = match texture.format() {
            wgpu::TextureFormat::Rgba8UnormSrgb => &self.pipeline_srgb,
            wgpu::TextureFormat::Rgba8Unorm => &self.pipeline_linear,
            other => {
                tracing::error!(
                    "MipGenerator: unsupported texture format {:?}, skipping mip generation",
                    other
                );
                return;
            }
        };

        let mip_count = texture.mip_level_count();
        if mip_count <= 1 {
            return;
        }

        for mip in 1..mip_count {
            let src_view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Mip Generator Source View"),
                format: Some(texture.format()),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: mip - 1,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: None,
                usage: None,
            });
            let dst_view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Mip Generator Destination View"),
                format: Some(texture.format()),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: mip,
                mip_level_count: Some(1),
                base_array_layer: layer,
                array_layer_count: Some(1),
                usage: None,
            });

            let params = MipParams {
                layer,
                padding_0: 0,
                padding_1: 0,
                padding_2: 0,
            };
            let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mip Generator Params"),
                size: std::mem::size_of::<MipParams>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: true,
            });
            params_buffer
                .slice(..)
                .get_mapped_range_mut()
                .copy_from_slice(bytemuck::bytes_of(&params));
            params_buffer.unmap();

            let bind_group = build::binding(
                device,
                "Mip Generator Bind Group",
                &self.bind_group_layout,
                vec![
                    wgpu::BindingResource::TextureView(&src_view),
                    wgpu::BindingResource::Sampler(&self.sampler),
                    params_buffer.as_entire_binding(),
                ],
            );

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mip Generator Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}
