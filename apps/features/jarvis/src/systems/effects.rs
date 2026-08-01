use crate::theme::{DRONE_COLOR, EFFECT_SECONDS, REPULSOR_COLOR};
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_scene::contract::Lifetime;

const KILL_PARTICLES: u32 = 700;
const HIT_PARTICLES: u32 = 90;

pub fn burst(storage: &mut nightshade::ecs::Storage, position: Vec3, kill: bool) {
    let flash = storage.spawn(());
    storage.set(flash, ParticleEmitter::flash_burst(position));
    storage.set(
        flash,
        Lifetime {
            remaining: EFFECT_SECONDS,
        },
    );

    let (color, count) = if kill {
        (
            Vec3::new(DRONE_COLOR[0], DRONE_COLOR[1], DRONE_COLOR[2]),
            KILL_PARTICLES,
        )
    } else {
        (
            Vec3::new(REPULSOR_COLOR[0], REPULSOR_COLOR[1], REPULSOR_COLOR[2]),
            HIT_PARTICLES,
        )
    };
    let explosion = storage.spawn(());
    storage.set(
        explosion,
        ParticleEmitter::firework_explosion(position, color, count),
    );
    storage.set(
        explosion,
        Lifetime {
            remaining: EFFECT_SECONDS,
        },
    );
}
