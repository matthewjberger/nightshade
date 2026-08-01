use crate::components::Effect;
use crate::data::EffectKind;
use crate::resources::Toggles;
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;

pub fn expire_effects(
    toggles: Res<Toggles>,
    mut commands: Commands,
    mut effects: Query<(&Effect, &mut ParticleEmitter)>,
) {
    effects.for_each_mut(|entity, (effect, emitter)| {
        let showing = match effect.0 {
            EffectKind::Fire => toggles.fire,
            EffectKind::Smoke => toggles.smoke,
            EffectKind::Sprite => true,
        };
        if showing {
            return;
        }
        emitter.enabled = false;
        commands.remove::<(Effect,)>(entity);
    });
}
