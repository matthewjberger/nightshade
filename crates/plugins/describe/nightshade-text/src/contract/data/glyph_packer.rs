use crate::contract::data::cosmic_text::{
    CacheKey, FontSystem, SubpixelBin, SwashCache, SwashContent,
};
use crate::contract::data::distance_field::{SUPERSAMPLE, distance_cell};
use nalgebra_glm::{Vec2, vec2};
use std::collections::HashMap;

pub const ATLAS_INITIAL_SIZE: u32 = 1024;
pub const ATLAS_MAX_SIZE: u32 = 8192;

const SHELF_PADDING: u32 = 1;

#[derive(Clone, Copy, Debug)]
pub struct GlyphAlloc {
    pub pixel_min: Vec2,
    pub pixel_max: Vec2,
    pub size: Vec2,
    pub placement_left: f32,
    pub placement_top: f32,
    pub is_color: bool,
}

pub struct RasterizedGlyph {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub placement_left: f32,
    pub placement_top: f32,
    pub is_color: bool,
}

pub enum Placement {
    Placed { x: u32, y: u32, alloc: GlyphAlloc },
    Full { next_size: Option<u32> },
}

struct Shelf {
    y: u32,
    height: u32,
    cursor_x: u32,
}

pub struct GlyphPacker {
    size: u32,
    revision: u32,
    glyphs: HashMap<CacheKey, Option<GlyphAlloc>>,
    shelves: Vec<Shelf>,
}

impl Default for GlyphPacker {
    fn default() -> Self {
        Self {
            size: ATLAS_INITIAL_SIZE,
            revision: 0,
            glyphs: HashMap::new(),
            shelves: Vec::new(),
        }
    }
}

impl GlyphPacker {
    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn revision(&self) -> u32 {
        self.revision
    }

    pub fn lookup(&self, cache_key: &CacheKey) -> Option<Option<GlyphAlloc>> {
        self.glyphs.get(&settled(*cache_key)).copied()
    }

    pub fn mark_empty(&mut self, cache_key: CacheKey) {
        self.glyphs.insert(settled(cache_key), None);
    }

    pub fn rasterize(
        &self,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        cache_key: CacheKey,
    ) -> Option<RasterizedGlyph> {
        let cache_key = settled(cache_key);
        let image = swash_cache.get_image_uncached(font_system, cache_key)?;
        let placement = image.placement;
        if placement.width == 0 || placement.height == 0 {
            return None;
        }

        match image.content {
            SwashContent::Mask => {
                let large_key = CacheKey {
                    font_size_bits: (f32::from_bits(cache_key.font_size_bits) * SUPERSAMPLE as f32)
                        .to_bits(),
                    ..cache_key
                };
                let large = swash_cache.get_image_uncached(font_system, large_key)?;
                let placement = large.placement;
                if placement.width == 0 || placement.height == 0 {
                    return None;
                }
                let cell = distance_cell(
                    &large.data,
                    placement.width,
                    placement.height,
                    placement.left,
                    placement.top,
                );
                let mut pixels = vec![255u8; (cell.width * cell.height * 4) as usize];
                for (index, alpha) in cell.alphas.iter().enumerate() {
                    pixels[index * 4 + 3] = *alpha;
                }
                Some(RasterizedGlyph {
                    width: cell.width,
                    height: cell.height,
                    pixels,
                    placement_left: cell.left as f32,
                    placement_top: cell.top as f32,
                    is_color: false,
                })
            }
            SwashContent::Color => {
                let width = placement.width;
                let height = placement.height;
                let length = (width * height * 4) as usize;
                Some(RasterizedGlyph {
                    width,
                    height,
                    pixels: image.data[..length].to_vec(),
                    placement_left: placement.left as f32,
                    placement_top: placement.top as f32,
                    is_color: true,
                })
            }
            SwashContent::SubpixelMask => {
                let width = placement.width;
                let height = placement.height;
                let mut pixels = vec![0u8; (width * height * 4) as usize];
                for index in 0..(width * height) as usize {
                    let red = image.data[index * 4];
                    let green = image.data[index * 4 + 1];
                    let blue = image.data[index * 4 + 2];
                    let target = index * 4;
                    pixels[target] = red;
                    pixels[target + 1] = green;
                    pixels[target + 2] = blue;
                    pixels[target + 3] = red.max(green).max(blue);
                }
                Some(RasterizedGlyph {
                    width,
                    height,
                    pixels,
                    placement_left: placement.left as f32,
                    placement_top: placement.top as f32,
                    is_color: true,
                })
            }
        }
    }

    pub fn place(&mut self, cache_key: CacheKey, glyph: &RasterizedGlyph) -> Placement {
        let cache_key = settled(cache_key);
        let Some((x, y)) = self.allocate(glyph.width, glyph.height) else {
            let next = (self.size * 2).min(ATLAS_MAX_SIZE);
            return Placement::Full {
                next_size: (next != self.size).then_some(next),
            };
        };

        let alloc = GlyphAlloc {
            pixel_min: vec2(x as f32, y as f32),
            pixel_max: vec2((x + glyph.width) as f32, (y + glyph.height) as f32),
            size: vec2(glyph.width as f32, glyph.height as f32),
            placement_left: glyph.placement_left,
            placement_top: glyph.placement_top,
            is_color: glyph.is_color,
        };
        self.glyphs.insert(cache_key, Some(alloc));
        Placement::Placed { x, y, alloc }
    }

    pub fn grew(&mut self, size: u32) {
        self.size = size;
        self.revision = self.revision.wrapping_add(1);
    }
}

fn settled(cache_key: CacheKey) -> CacheKey {
    CacheKey {
        x_bin: SubpixelBin::Zero,
        y_bin: SubpixelBin::Zero,
        ..cache_key
    }
}

impl GlyphPacker {
    fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        let needed_height = height + SHELF_PADDING * 2;
        let needed_width = width + SHELF_PADDING;

        for shelf in &mut self.shelves {
            if shelf.height >= height
                && shelf.cursor_x + needed_width <= self.size
                && shelf.height as f32 <= height as f32 * 1.5
            {
                let position = (shelf.cursor_x, shelf.y);
                shelf.cursor_x += needed_width;
                return Some(position);
            }
        }

        let next_shelf_y = self
            .shelves
            .last()
            .map(|shelf| shelf.y + shelf.height + SHELF_PADDING)
            .unwrap_or(SHELF_PADDING);

        if next_shelf_y + needed_height > self.size || needed_width > self.size {
            return None;
        }

        self.shelves.push(Shelf {
            y: next_shelf_y,
            height,
            cursor_x: SHELF_PADDING + needed_width,
        });
        Some((SHELF_PADDING, next_shelf_y))
    }
}

pub enum GlyphWork<'a> {
    Grow {
        previous: u32,
        next: u32,
    },
    Upload {
        x: u32,
        y: u32,
        glyph: &'a RasterizedGlyph,
    },
}

pub fn ensure_glyph_slot(
    packer: &mut GlyphPacker,
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    cache_key: CacheKey,
    store: &mut dyn FnMut(GlyphWork) -> bool,
) -> Option<GlyphAlloc> {
    if let Some(slot) = packer.lookup(&cache_key) {
        return slot;
    }

    let Some(glyph) = packer.rasterize(font_system, swash_cache, cache_key) else {
        packer.mark_empty(cache_key);
        return None;
    };

    let placed = match packer.place(cache_key, &glyph) {
        Placement::Placed { x, y, alloc } => Some((x, y, alloc)),
        Placement::Full { next_size } => {
            let next_size = next_size?;
            let grown = store(GlyphWork::Grow {
                previous: packer.size(),
                next: next_size,
            });
            if !grown {
                return None;
            }
            packer.grew(next_size);
            match packer.place(cache_key, &glyph) {
                Placement::Placed { x, y, alloc } => Some((x, y, alloc)),
                Placement::Full { .. } => None,
            }
        }
    };

    let (x, y, alloc) = placed?;
    if !store(GlyphWork::Upload {
        x,
        y,
        glyph: &glyph,
    }) {
        return None;
    }
    Some(alloc)
}
