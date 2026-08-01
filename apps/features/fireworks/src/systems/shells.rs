use crate::commands::explode;
use crate::components::{Burst, Fuse, Generation, SecondTint, Tint, Velocity};
use crate::data::ExplosionType;
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_platform::prelude::*;

pub fn advance_shells(
    time: Res<Time>,
    mut commands: Commands,
    mut shells: Query<(
        &mut Fuse,
        &mut Velocity,
        &Tint,
        &SecondTint,
        &Burst,
        &mut ParticleEmitter,
    )>,
) {
    let delta_time = time.delta_time;

    shells.for_each_mut(
        |entity, (fuse, velocity, tint, second_tint, burst, emitter)| {
            fuse.0 -= delta_time;
            emitter.position += velocity.0 * delta_time;
            velocity.0.y -= 9.81 * delta_time;

            if fuse.0 > 0.0 {
                return;
            }

            let position = emitter.position;
            emitter.enabled = false;
            commands.remove::<(Fuse, Velocity, Tint, SecondTint, Burst, Generation)>(entity);

            explode::spawn_explosion(&mut commands, position, tint.0, second_tint.0, burst.0);
            if burst.0 == ExplosionType::SplittingShell {
                explode::spawn_splitting_sub_shells(&mut commands, position, tint.0, second_tint.0);
            }
        },
    );
}
