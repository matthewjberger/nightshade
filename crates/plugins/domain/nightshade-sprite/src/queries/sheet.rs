use crate::contract::{SpriteRegion, TileTrim};
use nalgebra_glm::Vec2;

pub fn tile_pixels(rgba: &[u8], width: u32, side: u32, columns: u32, index: u32) -> Vec<u8> {
    let columns = columns.max(1);
    let column = index % columns;
    let row = index / columns;
    let mut pixels = Vec::with_capacity((side * side * 4) as usize);
    for y in 0..side {
        for x in 0..side {
            let texel = (((row * side + y) * width + column * side + x) * 4) as usize;
            match rgba.get(texel..texel + 4) {
                Some(color) => pixels.extend_from_slice(color),
                None => pixels.extend_from_slice(&[255, 255, 255, 255]),
            }
        }
    }
    pixels
}

pub fn painted_share(trim: TileTrim, side: u32) -> f32 {
    let side = side.max(1) as f32;
    ((side - trim.above as f32 - trim.below as f32) / side).clamp(0.0, 1.0)
}

pub fn standing_tile(cell: SpriteRegion, trim: TileTrim, side: u32) -> (SpriteRegion, f32) {
    let texels = side.max(1) as f32;
    let span = cell.uv_max.y - cell.uv_min.y;
    let region = SpriteRegion::new(
        Vec2::new(
            cell.uv_min.x,
            cell.uv_min.y + span * trim.above as f32 / texels,
        ),
        Vec2::new(
            cell.uv_max.x,
            cell.uv_max.y - span * trim.below as f32 / texels,
        ),
    );
    (region, painted_share(trim, side))
}

pub fn standing_span(
    left: SpriteRegion,
    right: SpriteRegion,
    left_trim: TileTrim,
    right_trim: TileTrim,
    side: u32,
) -> (SpriteRegion, f32) {
    let trim = TileTrim {
        above: left_trim.above.min(right_trim.above),
        below: left_trim.below.min(right_trim.below),
    };
    let (cut, share) = standing_tile(left, trim, side);
    (
        SpriteRegion::new(cut.uv_min, Vec2::new(right.uv_max.x, cut.uv_max.y)),
        share,
    )
}
