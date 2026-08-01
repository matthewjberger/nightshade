use crate::contract::data::ParticleTextureUpload;

#[derive(Default)]
pub struct ParticleTextures {
    pub pending: Vec<ParticleTextureUpload>,
}

#[derive(Default)]
pub struct ParticleFrame {
    pub emitters: Vec<(nightshade_ecs::Entity, crate::contract::ParticleEmitter)>,
}
