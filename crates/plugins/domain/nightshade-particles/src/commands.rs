use crate::contract::{ParticleTextureUpload, ParticleTextures};

pub fn queue_particle_texture(textures: &mut ParticleTextures, upload: ParticleTextureUpload) {
    textures.pending.push(upload);
}
