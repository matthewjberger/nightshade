use crate::data::Rgba;
use crate::queries::paint;

pub fn shingles(side: u32, tile: Rgba, shade: Rgba) -> Vec<u8> {
    paint(side, |x, y| {
        let course = y / 4;
        let shift = if course.is_multiple_of(2) { 0 } else { 2 };
        if y % 4 == 3 || (x + shift).is_multiple_of(4) && y % 4 != 0 {
            shade
        } else {
            tile
        }
    })
}
