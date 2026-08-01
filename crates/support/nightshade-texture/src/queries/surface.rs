use crate::data::Rgba;
use crate::queries::paint;
use fastrand::Rng;

pub fn speckled(
    side: u32,
    base: Rgba,
    dark: Rgba,
    light: Rgba,
    seed: u64,
    dark_share: u32,
    light_share: u32,
) -> Vec<u8> {
    let mut rng = Rng::with_seed(seed);
    paint(side, |_, _| {
        let roll = rng.u32(..) % 100;
        if roll < dark_share {
            dark
        } else if roll < dark_share + light_share {
            light
        } else {
            base
        }
    })
}

pub fn cobbles(side: u32, stone: Rgba, alt: Rgba, mortar: Rgba, seed: u64) -> Vec<u8> {
    let mut rng = Rng::with_seed(seed);
    paint(side, |x, y| {
        if x.is_multiple_of(4) || y.is_multiple_of(4) {
            mortar
        } else if rng.u32(..).is_multiple_of(3) {
            alt
        } else {
            stone
        }
    })
}

pub fn stone_wall(side: u32, stone: Rgba, alt: Rgba, mortar: Rgba, seed: u64) -> Vec<u8> {
    let mut rng = Rng::with_seed(seed);
    paint(side, |x, y| {
        let course = y / 4;
        let shift = if course.is_multiple_of(2) { 0 } else { 4 };
        if y.is_multiple_of(4) || (x + shift).is_multiple_of(8) {
            mortar
        } else if rng.u32(..).is_multiple_of(5) {
            alt
        } else {
            stone
        }
    })
}

pub fn checkers(side: u32, cell: u32, light: Rgba, dark: Rgba) -> Vec<u8> {
    paint(side, |x, y| {
        if ((x / cell) + (y / cell)).is_multiple_of(2) {
            light
        } else {
            dark
        }
    })
}
