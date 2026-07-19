//! Shelf-packed glyph atlas backing text rendering.

use cosmic_text::{CacheKey, FontSystem, SwashCache, SwashContent};
use nalgebra_glm::{Vec2, vec2};
use std::collections::HashMap;

const ATLAS_INITIAL_SIZE: u32 = 1024;
const ATLAS_MAX_SIZE: u32 = 8192;
const SHELF_PADDING: u32 = 1;

/// Placement of a single rasterized glyph within the atlas texture.
#[derive(Clone, Copy, Debug)]
pub struct GlyphAlloc {
    /// Top-left UV coordinate of the glyph in the atlas.
    pub uv_min: Vec2,
    /// Bottom-right UV coordinate of the glyph in the atlas.
    pub uv_max: Vec2,
    /// Glyph size in pixels.
    pub size: Vec2,
    /// Horizontal bearing from the pen position in pixels.
    pub placement_left: f32,
    /// Vertical bearing from the pen baseline in pixels.
    pub placement_top: f32,
    /// Whether the glyph is a color bitmap rather than a coverage mask.
    pub is_color: bool,
}

struct Shelf {
    y: u32,
    height: u32,
    cursor_x: u32,
}

/// Shelf-packed texture atlas caching rasterized glyphs, growing on demand.
pub struct GlyphAtlas {
    /// The atlas texture.
    pub texture: wgpu::Texture,
    /// View over the atlas texture.
    pub texture_view: wgpu::TextureView,
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// Counter bumped whenever the atlas texture is reallocated.
    pub revision: u32,
    glyphs: HashMap<CacheKey, Option<GlyphAlloc>>,
    shelves: Vec<Shelf>,
}

impl GlyphAtlas {
    /// Creates an atlas at the initial size with a fresh texture.
    pub fn new(device: &wgpu::Device) -> Self {
        let (texture, texture_view) = create_atlas_texture(device, ATLAS_INITIAL_SIZE);
        Self {
            texture,
            texture_view,
            width: ATLAS_INITIAL_SIZE,
            height: ATLAS_INITIAL_SIZE,
            revision: 0,
            glyphs: HashMap::new(),
            shelves: Vec::new(),
        }
    }

    /// Returns the cached allocation for `cache_key`, rasterizing and uploading it on first use.
    ///
    /// Returns `None` when the glyph has no pixels or the atlas cannot grow to fit it.
    pub fn ensure_glyph(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        cache_key: CacheKey,
    ) -> Option<GlyphAlloc> {
        if let Some(slot) = self.glyphs.get(&cache_key) {
            return *slot;
        }

        let image = swash_cache.get_image_uncached(font_system, cache_key)?;
        let placement = image.placement;

        if placement.width == 0 || placement.height == 0 {
            self.glyphs.insert(cache_key, None);
            return None;
        }

        let glyph_width = placement.width;
        let glyph_height = placement.height;

        let alloc_pos = self.allocate(glyph_width, glyph_height);
        let alloc_pos = match alloc_pos {
            Some(pos) => pos,
            None => {
                if !self.try_grow(device, queue) {
                    return None;
                }
                self.allocate(glyph_width, glyph_height)?
            }
        };

        let is_color = matches!(image.content, SwashContent::Color);
        let mut staging = vec![0u8; (glyph_width * glyph_height * 4) as usize];
        match image.content {
            SwashContent::Mask => {
                for index in 0..(glyph_width * glyph_height) as usize {
                    let coverage = image.data[index];
                    let dst = index * 4;
                    staging[dst] = 255;
                    staging[dst + 1] = 255;
                    staging[dst + 2] = 255;
                    staging[dst + 3] = coverage;
                }
            }
            SwashContent::Color => {
                let len = staging.len();
                staging.copy_from_slice(&image.data[..len]);
            }
            SwashContent::SubpixelMask => {
                for index in 0..(glyph_width * glyph_height) as usize {
                    let red = image.data[index * 4];
                    let green = image.data[index * 4 + 1];
                    let blue = image.data[index * 4 + 2];
                    let dst = index * 4;
                    staging[dst] = red;
                    staging[dst + 1] = green;
                    staging[dst + 2] = blue;
                    staging[dst + 3] = red.max(green).max(blue);
                }
            }
        }

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: alloc_pos.0,
                    y: alloc_pos.1,
                    z: 0,
                },
            },
            &staging,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(glyph_width * 4),
                rows_per_image: Some(glyph_height),
            },
            wgpu::Extent3d {
                width: glyph_width,
                height: glyph_height,
                depth_or_array_layers: 1,
            },
        );

        let alloc = GlyphAlloc {
            uv_min: vec2(
                alloc_pos.0 as f32 / self.width as f32,
                alloc_pos.1 as f32 / self.height as f32,
            ),
            uv_max: vec2(
                (alloc_pos.0 + glyph_width) as f32 / self.width as f32,
                (alloc_pos.1 + glyph_height) as f32 / self.height as f32,
            ),
            size: vec2(glyph_width as f32, glyph_height as f32),
            placement_left: placement.left as f32,
            placement_top: placement.top as f32,
            is_color,
        };

        self.glyphs.insert(cache_key, Some(alloc));
        Some(alloc)
    }

    fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        let needed_height = height + SHELF_PADDING * 2;
        let needed_width = width + SHELF_PADDING;

        for shelf in &mut self.shelves {
            if shelf.height >= height
                && shelf.cursor_x + needed_width <= self.width
                && shelf.height as f32 <= height as f32 * 1.5
            {
                let pos = (shelf.cursor_x, shelf.y);
                shelf.cursor_x += needed_width;
                return Some(pos);
            }
        }

        let next_shelf_y = self
            .shelves
            .last()
            .map(|s| s.y + s.height + SHELF_PADDING)
            .unwrap_or(SHELF_PADDING);

        if next_shelf_y + needed_height > self.height || needed_width > self.width {
            return None;
        }

        let shelf = Shelf {
            y: next_shelf_y,
            height,
            cursor_x: SHELF_PADDING + needed_width,
        };
        let pos = (SHELF_PADDING, next_shelf_y);
        self.shelves.push(shelf);
        Some(pos)
    }

    fn try_grow(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> bool {
        let new_size = (self.width * 2).min(ATLAS_MAX_SIZE);
        if new_size == self.width {
            return false;
        }

        let (new_texture, new_view) = create_atlas_texture(device, new_size);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Glyph Atlas Grow"),
        });
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
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
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(std::iter::once(encoder.finish()));

        let old_width = self.width;
        let old_height = self.height;
        self.texture = new_texture;
        self.texture_view = new_view;
        self.width = new_size;
        self.height = new_size;
        self.revision = self.revision.wrapping_add(1);

        let scale_x = old_width as f32 / new_size as f32;
        let scale_y = old_height as f32 / new_size as f32;
        for slot in self.glyphs.values_mut() {
            if let Some(alloc) = slot.as_mut() {
                alloc.uv_min.x *= scale_x;
                alloc.uv_min.y *= scale_y;
                alloc.uv_max.x *= scale_x;
                alloc.uv_max.y *= scale_y;
            }
        }
        true
    }
}

fn create_atlas_texture(device: &wgpu::Device, size: u32) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Glyph Atlas"),
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
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
