use nightshade_ease::{lerp, smooth_step};
use nightshade_noise::unit;

use crate::data;

pub fn pixels() -> Vec<u8> {
    let side = data::GROUND_TEXTURE_SIDE;
    let mut rgba = Vec::with_capacity((side * side * 4) as usize);
    for down in 0..side {
        for across in 0..side {
            let x = across as f32 / side as f32;
            let y = down as f32 / side as f32;
            let broad = layered(x, y, 3, 3, 0.55, 17);
            let patch = layered(x, y, 7, 3, 0.5, 53);
            let grain = layered(x, y, 40, 2, 0.5, 91);
            let speck = layered(x, y, 96, 1, 0.5, 131);

            let soil = mix(data::GROUND_SOIL, data::GROUND_MOSS, smooth_step(patch));
            let stone = mix(soil, data::GROUND_STONE, smooth_step(broad * 1.15 - 0.15));
            let shade = 0.94 + (grain - 0.5) * 0.16;
            let speckled = [stone[0] * shade, stone[1] * shade, stone[2] * shade];
            let lit = mix(
                speckled,
                data::GROUND_PEBBLE,
                smooth_step((speck - 0.74) / 0.22) * 0.45,
            );

            rgba.push(channel(lit[0]));
            rgba.push(channel(lit[1]));
            rgba.push(channel(lit[2]));
            rgba.push(255);
        }
    }
    rgba
}

fn layered(x: f32, y: f32, period: i32, layers: u32, fade: f32, seed: u32) -> f32 {
    let mut sum = 0.0;
    let mut weight = 1.0;
    let mut total = 0.0;
    let mut cells = period;
    for layer in 0..layers.max(1) {
        sum += tiled(x * cells as f32, y * cells as f32, cells, seed + layer) * weight;
        total += weight;
        weight *= fade;
        cells *= 2;
    }
    if total <= 0.0 {
        return 0.0;
    }
    sum / total
}

fn tiled(across: f32, down: f32, period: i32, seed: u32) -> f32 {
    let left = across.floor();
    let top = down.floor();
    let inside_across = smooth_step(across - left);
    let inside_down = smooth_step(down - top);
    let wrap = |value: f32| (value as i32).rem_euclid(period.max(1));
    let one = unit(wrap(left), wrap(top), seed);
    let other = unit(wrap(left + 1.0), wrap(top), seed);
    let third = unit(wrap(left), wrap(top + 1.0), seed);
    let fourth = unit(wrap(left + 1.0), wrap(top + 1.0), seed);
    let upper = one + (other - one) * inside_across;
    let lower = third + (fourth - third) * inside_across;
    upper + (lower - upper) * inside_down
}

fn mix(from: [f32; 3], to: [f32; 3], amount: f32) -> [f32; 3] {
    [
        lerp(from[0], to[0], amount),
        lerp(from[1], to[1], amount),
        lerp(from[2], to[2], amount),
    ]
}

fn channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
