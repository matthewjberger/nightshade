use crate::contract::{
    AcousticEmitter, AcousticListener, AcousticsFrame, EmitterPlace, MAX_EMITTERS,
};
use nightshade::prelude::*;
use nightshade_scene::contract::GlobalTransform;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn gather_acoustics_system(
    mut frame: ResMut<AcousticsFrame>,
    listeners: Query<(&AcousticListener, &GlobalTransform)>,
    emitters: Query<(&AcousticEmitter, &GlobalTransform)>,
) {
    frame.heard = false;
    listeners.for_each(|_entity, (_listener, transform)| {
        if frame.heard {
            return;
        }
        frame.listener = place_of(transform);
        frame.heard = true;
    });

    frame.emitters.clear();
    frame.leeches.clear();
    let mut asked = 0usize;
    emitters.for_each(|entity, (emitter, transform)| {
        if !emitter.sounding {
            return;
        }
        asked += 1;
        if frame.emitters.len() >= MAX_EMITTERS {
            return;
        }
        if let Some(parent) = emitter.leech_of {
            frame.leeches.push((entity, parent));
            return;
        }
        frame.emitters.push(EmitterPlace {
            entity,
            position: place_of(transform),
            reach: emitter.reach,
            voice: emitter.voice,
            hold: emitter.hold,
            own_reverb: emitter.own_reverb,
            listener_reverb: emitter.listener_reverb,
            reverb_rays: emitter.reverb_rays,
            reverb_bounces: emitter.reverb_bounces,
            occlusion_rays: emitter.occlusion_rays,
            occlusion_bounces: emitter.occlusion_bounces,
            permeation_rays: emitter.permeation_rays,
            permeation_bounces: emitter.permeation_bounces,
            ambient_occlusion_rays: emitter.ambient_occlusion_rays,
            ambient_occlusion_bounces: emitter.ambient_occlusion_bounces,
            ambient_permeation_rays: emitter.ambient_permeation_rays,
            ambient_permeation_bounces: emitter.ambient_permeation_bounces,
            bucket_seconds: emitter.bucket_seconds,
            seed: emitter.seed,
            clamped: emitter.clamped,
        });
    });

    if asked > MAX_EMITTERS && !TOLD.swap(true, Ordering::Relaxed) {
        tracing::warn!(
            "{asked} acoustic emitters is more than the {MAX_EMITTERS} the rays carry, so the rest are left out"
        );
    }
}

static TOLD: AtomicBool = AtomicBool::new(false);

fn place_of(transform: &GlobalTransform) -> Vec3 {
    Vec3::new(
        transform.0[(0, 3)],
        transform.0[(1, 3)],
        transform.0[(2, 3)],
    )
}
