use crate::data::Rgba;
use crate::queries::paint;
use fastrand::Rng;

pub fn planks(side: u32, board: Rgba, grain: Rgba, seam: Rgba, seed: u64) -> Vec<u8> {
    let mut rng = Rng::with_seed(seed);
    paint(side, |x, y| {
        if y.is_multiple_of(4) || (x + y / 4 * 5).is_multiple_of(8) && y % 4 == 2 {
            seam
        } else if rng.u32(..).is_multiple_of(7) {
            grain
        } else {
            board
        }
    })
}

pub fn furrows(side: u32, earth: Rgba, ridge: Rgba, seed: u64) -> Vec<u8> {
    let mut rng = Rng::with_seed(seed);
    paint(side, |_, y| {
        let on_ridge = (y + 1).is_multiple_of(4);
        if on_ridge && !rng.u32(..).is_multiple_of(5) {
            ridge
        } else {
            earth
        }
    })
}

pub fn door(side: u32, face: Rgba, dark: Rgba, knob: Rgba) -> Vec<u8> {
    paint(side, |x, y| {
        if x == 0 || x == side - 1 || y == 0 {
            dark
        } else if (x == 11 || x == 12) && (y == 8 || y == 9) {
            knob
        } else if (x + 1).is_multiple_of(4) {
            dark
        } else {
            face
        }
    })
}
