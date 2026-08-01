use crate::data;
use nightshade::prelude::*;
use nightshade_particles::contract::{ParticleTextureUpload, ParticleTextures};

pub fn dress(mut textures: ResMut<ParticleTextures>) {
    let width = data::SPARK_SPRITE;
    let middle = width as f32 * 0.5;
    let mut pixels = Vec::with_capacity((width * width * 4) as usize);
    for row in 0..width {
        for column in 0..width {
            let reach = ((column as f32 - middle).hypot(row as f32 - middle)) / middle;
            let alpha = (1.0 - reach).clamp(0.0, 1.0).powf(1.6);
            pixels.extend_from_slice(&[255, 255, 255, (alpha * 255.0) as u8]);
        }
    }
    textures.pending.push(ParticleTextureUpload {
        slot: 0,
        rgba_data: pixels,
        width,
        height: width,
    });
}
