use crate::contract::ParticleEmitter;

pub fn update_particle_emitters(
    time: nightshade::ecs::system_param::Res<nightshade_platform::contract::time::Time>,
    emitters: nightshade::ecs::system_param::Query<(&mut ParticleEmitter,)>,
) {
    let delta_time = time.delta_time;
    emitters.for_each(|_entity, (emitter,)| {
        if !emitter.enabled || (emitter.one_shot && emitter.has_fired) {
            return;
        }
        if emitter.burst_count > 0 && !emitter.has_fired {
            emitter.accumulated_spawn = emitter.burst_count as f32;
            emitter.has_fired = true;
        } else if emitter.spawn_rate > 0.0 {
            emitter.accumulated_spawn += emitter.spawn_rate * delta_time;
        }
    });
}
