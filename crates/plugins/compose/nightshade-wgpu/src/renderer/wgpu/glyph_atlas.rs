use nightshade_text::contract::data::cosmic_text::{CacheKey, FontSystem, SwashCache};
use nightshade_text::contract::data::glyph_packer::{
    ATLAS_INITIAL_SIZE, GlyphPacker, GlyphWork, RasterizedGlyph, ensure_glyph_slot,
};

pub use nightshade_text::contract::data::glyph_packer::GlyphAlloc;

pub struct GlyphAtlas {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    packer: GlyphPacker,
}

impl GlyphAtlas {
    pub fn new(device: &wgpu::Device) -> Self {
        let (texture, texture_view) = create_atlas_texture(device, ATLAS_INITIAL_SIZE);
        Self {
            texture,
            texture_view,
            packer: GlyphPacker::default(),
        }
    }

    pub fn revision(&self) -> u32 {
        self.packer.revision()
    }

    pub fn ensure_glyph(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        cache_key: CacheKey,
    ) -> Option<GlyphAlloc> {
        let GlyphAtlas {
            texture,
            texture_view,
            packer,
        } = self;
        ensure_glyph_slot(
            packer,
            font_system,
            swash_cache,
            cache_key,
            &mut |work| match work {
                GlyphWork::Grow { previous, next } => {
                    grow_texture(texture, texture_view, device, queue, previous, next);
                    true
                }
                GlyphWork::Upload { x, y, glyph } => {
                    upload_glyph(texture, queue, x, y, glyph);
                    true
                }
            },
        )
    }
}

fn upload_glyph(
    texture: &wgpu::Texture,
    queue: &wgpu::Queue,
    x: u32,
    y: u32,
    glyph: &RasterizedGlyph,
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            aspect: wgpu::TextureAspect::All,
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x, y, z: 0 },
        },
        &glyph.pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(glyph.width * 4),
            rows_per_image: Some(glyph.height),
        },
        wgpu::Extent3d {
            width: glyph.width,
            height: glyph.height,
            depth_or_array_layers: 1,
        },
    );
}

fn grow_texture(
    texture: &mut wgpu::Texture,
    texture_view: &mut wgpu::TextureView,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    previous: u32,
    size: u32,
) {
    let (new_texture, new_view) = create_atlas_texture(device, size);

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Glyph Atlas Grow"),
    });
    encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyTextureInfo {
            texture: &new_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width: previous,
            height: previous,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(std::iter::once(encoder.finish()));

    *texture = new_texture;
    *texture_view = new_view;
}

fn create_atlas_texture(device: &wgpu::Device, size: u32) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Glyph Atlas"),
        size: crate::renderer::texture_extent::texture_extent(device, size, size, 1),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}
