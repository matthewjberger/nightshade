use crate::contract::{ParticleEmitter, ParticleFrame};
use nightshade::prelude::*;

pub fn gather_emitters_system(
    mut frame: ResMut<ParticleFrame>,
    sources: Query<(&ParticleEmitter,)>,
) {
    frame.emitters.clear();
    sources.for_each(|entity, (emitter,)| {
        frame.emitters.push((entity, emitter.clone()));
    });
}
