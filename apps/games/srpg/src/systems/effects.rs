use crate::tags::Firefly;

use crate::theme;
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_render::data::config::RendererState;
use nightshade_render::prelude::night_factor;

pub fn tend_the_fireflies(
    renderer_state: Res<RendererState>,
    fireflies: Query<(&mut ParticleEmitter,), WithTag<Firefly>>,
) {
    let out = night_factor(renderer_state.day_night.hour, &theme::TWILIGHT) > 0.5;
    fireflies.for_each(|_entity, (emitter,)| {
        emitter.enabled = out;
    });
}
