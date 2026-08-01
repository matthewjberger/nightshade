use crate::commands::forget;
use crate::components::{Alert, Exposure, Hearing, LastSeen, Sight};
use crate::queries::{loudest, visible};
use crate::resources::{Noises, SensesSettings};
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;
use nightshade_scene::contract::LocalTransform;

pub fn notice_the_hiders(
    time: Res<Time>,
    settings: Res<SensesSettings>,
    noises: Res<Noises>,
    mut hiders: Query<(&Exposure, &LocalTransform)>,
    listeners: Query<(&Hearing, &Sight, &mut Alert, &mut LastSeen, &LocalTransform)>,
) {
    let delta = time.delta_time;
    let mut hidden: Vec<(Vec3, Exposure)> = Vec::new();
    hiders.for_each_mut(|_entity, (exposure, transform)| {
        hidden.push((transform.translation, *exposure));
    });
    listeners.for_each(|_entity, (hearing, sight, alert, seen, transform)| {
        let here = transform.translation;
        let mut noticed = false;
        for (target, exposure) in &hidden {
            if visible(
                &settings,
                here,
                &transform.rotation,
                *target,
                sight,
                exposure,
            ) {
                seen.0 = *target;
                alert.0 = 1.0;
                noticed = true;
            }
        }
        if noticed {
            return;
        }
        if let Some(noise) = loudest(&noises, &settings, here, hearing) {
            seen.0 = noise.at;
            alert.0 = (alert.0 + settings.noise_gain * noise.loudness).clamp(0.0, 1.0);
            return;
        }
        alert.0 = (alert.0 - settings.calm * delta).max(0.0);
    });
}

pub fn clear_the_noises(mut noises: ResMut<Noises>) {
    forget(&mut noises);
}
