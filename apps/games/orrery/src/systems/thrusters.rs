use crate::data;
use crate::queries;
use crate::resources::{Mode, OrreryResources};
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;

const PLUME_RATE: f32 = 220.0;
const PLUME_SPEED: f32 = 7.0;
const PLUME_OFFSET: f32 = 1.3;

pub fn blow(resources: Res<OrreryResources>, mut emitters: Query<(&mut ParticleEmitter,)>) {
    let plume = resources.scene.plume;
    if plume == Entity::default() {
        return;
    }

    let pose = resources.pose;
    let firing = resources.flight.mode == Mode::Flying
        && pose.landed.is_none()
        && pose.thrust.magnitude() > 0.1;
    let exhaust = queries::normalized_or(-pose.thrust, Vec3::z());
    let strength = (pose.thrust.magnitude() / data::SHIP_MAIN_THRUST).clamp(0.0, 1.0);

    emitters.for_entity(plume, |(emitter,)| {
        emitter.enabled = firing;
        emitter.spawn_rate = if firing { PLUME_RATE * strength } else { 0.0 };
        emitter.position = pose.ship_position + exhaust * PLUME_OFFSET;
        emitter.direction = exhaust;
        emitter.initial_velocity_min = PLUME_SPEED * 0.4 * strength;
        emitter.initial_velocity_max = PLUME_SPEED * strength;
    });
}
