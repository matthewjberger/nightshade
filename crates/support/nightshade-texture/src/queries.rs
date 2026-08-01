pub mod blob;
pub mod glass;
pub mod roof;
pub mod surface;
pub mod wood;

use crate::data::Rgba;

pub fn paint(side: u32, mut color_at: impl FnMut(u32, u32) -> Rgba) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((side * side * 4) as usize);
    for y in 0..side {
        for x in 0..side {
            pixels.extend_from_slice(&color_at(x, y));
        }
    }
    pixels
}
