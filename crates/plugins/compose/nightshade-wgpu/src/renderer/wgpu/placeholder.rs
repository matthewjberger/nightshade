use crate::renderer::wgpu::build;
pub fn placeholder_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    format: wgpu::TextureFormat,
    layers: u32,
    texel: &[u8],
) -> wgpu::Texture {
    let size = wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: layers,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let data = texel.repeat(layers as usize);
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(texel.len() as u32),
            rows_per_image: Some(1),
        },
        size,
    );

    texture
}

pub fn placeholder_depth_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
) -> wgpu::Texture {
    let texture = build::surface(
        device,
        label,
        wgpu::TextureFormat::Depth32Float,
        [1, 1, 1],
        1,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
    drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    }));
    queue.submit(Some(encoder.finish()));

    texture
}

pub const TRANSPARENT_RGBA16: [u8; 8] = [0; 8];

pub const TRANSPARENT_RGBA8: [u8; 4] = [0; 4];

pub const STILL_RG16: [u8; 4] = [0; 4];

pub const FAR_R32: [u8; 4] = 1.0f32.to_le_bytes();

pub const NEAR_NOTHING_R32: [u8; 4] = 0.0f32.to_le_bytes();
