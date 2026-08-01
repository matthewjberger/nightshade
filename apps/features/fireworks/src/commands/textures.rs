use crate::data::PARTICLE_TEXTURES;
use nightshade_particles::contract::{ParticleTextureUpload, ParticleTextures};
use nightshade_render::data::decoded_image::decode_to_rgba8;

pub fn load_particle_textures(textures: &mut ParticleTextures) {
    for &(slot, bytes) in PARTICLE_TEXTURES {
        let image = decode_to_rgba8(bytes).expect("Failed to decode particle texture");
        textures.pending.push(ParticleTextureUpload {
            slot,
            rgba_data: image.rgba,
            width: image.width,
            height: image.height,
        });
    }
}
