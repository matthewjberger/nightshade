use crate::data;
use nightshade::prelude::*;
use nightshade_sprite::prelude::{
    SpriteAtlases, SpriteFilter, sprite_atlas_create, sprite_atlas_insert,
};

pub fn dress(mut atlases: ResMut<SpriteAtlases>) {
    sprite_atlas_create(
        &mut atlases,
        data::SHEET_ATLAS,
        data::SHEET_ATLAS_SIDE,
        SpriteFilter::Nearest,
    );
    let side = data::SHEET_SIDE;
    sprite_atlas_insert(
        &mut atlases,
        data::SHEET_ATLAS,
        data::SHEET_TILES,
        &tiles(),
        side * data::SHEET_ACROSS,
        side * data::SHEET_ACROSS,
    );
    sprite_atlas_insert(
        &mut atlases,
        data::SHEET_ATLAS,
        data::SHEET_FACE,
        &face(),
        side,
        side,
    );
}

fn tiles() -> Vec<u8> {
    let side = data::SHEET_SIDE;
    let across = data::SHEET_ACROSS;
    let wide = side * across;
    let tones: [[u8; 3]; 16] = [
        [64, 112, 72],
        [86, 138, 88],
        [120, 158, 92],
        [156, 176, 104],
        [180, 166, 118],
        [196, 150, 106],
        [172, 122, 88],
        [138, 100, 78],
        [110, 118, 148],
        [92, 132, 176],
        [120, 168, 196],
        [160, 196, 214],
        [198, 206, 220],
        [150, 150, 164],
        [104, 104, 118],
        [70, 70, 84],
    ];
    let mut pixels = vec![0u8; (wide * wide * 4) as usize];
    for cell in 0..across * across {
        let column = cell % across;
        let row = cell / across;
        let tone = tones[(cell as usize) % tones.len()];
        for down in 0..side {
            for along in 0..side {
                let edge = down == 0 || along == 0 || down + 1 == side || along + 1 == side;
                let grain = (((down * 5 + along * 3) % 7) as i32 - 3) * 4;
                let at = (((row * side + down) * wide + column * side + along) * 4) as usize;
                for part in 0..3 {
                    let base = tone[part] as i32 + if edge { -34 } else { grain };
                    pixels[at + part] = base.clamp(0, 255) as u8;
                }
                pixels[at + 3] = 255;
            }
        }
    }
    pixels
}

fn face() -> Vec<u8> {
    let side = data::SHEET_SIDE;
    let middle = side as f32 * 0.5;
    let mut pixels = Vec::with_capacity((side * side * 4) as usize);
    for down in 0..side {
        for along in 0..side {
            let reach = ((along as f32 - middle).hypot(down as f32 - middle)) / middle;
            let inside = reach < 0.9;
            let eye =
                down > side / 4 && down < side / 2 && (along as i32 - (side as i32 / 3)).abs() < 3
                    || down > side / 4
                        && down < side / 2
                        && (along as i32 - (2 * side as i32 / 3)).abs() < 3;
            let mouth = down > 2 * side / 3 && down < 3 * side / 4 && reach < 0.55;
            let level = match (inside, eye || mouth) {
                (true, true) => [30, 26, 40, 255],
                (true, false) => [242, 208, 150, 255],
                _ => [0, 0, 0, 0],
            };
            pixels.extend_from_slice(&level);
        }
    }
    pixels
}
