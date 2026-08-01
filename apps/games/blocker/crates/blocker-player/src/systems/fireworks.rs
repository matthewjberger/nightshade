use crate::resources::Fireworks;
use crate::theme;
use nightshade::prelude::*;
use nightshade_particles::contract::ParticleEmitter;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::Lifetime;

pub fn fly_the_shells(
    time: Res<Time>,
    mut fireworks: ResMut<Fireworks>,
    mut commands: Commands,
    mut trails: Query<(&mut ParticleEmitter,)>,
) {
    let delta_time = time.delta_time;
    let mut bursts = Vec::new();
    for shell in &mut fireworks.shells {
        shell.fuse -= delta_time;
        shell.velocity.y -= theme::SHELL_GRAVITY * delta_time;
        shell.position += shell.velocity * delta_time;
        let position = shell.position;
        let done = shell.fuse <= 0.0;
        trails.for_entity(shell.entity, |(emitter,)| {
            emitter.position = position;
            if done {
                emitter.enabled = false;
            }
        });
        if done {
            bursts.push((position, shell.color));
        }
    }
    fireworks.shells.retain(|shell| shell.fuse > 0.0);

    for (position, color) in bursts {
        let flash = commands.spawn(());
        let mut flash_emitter = ParticleEmitter::flash_burst(position);
        flash_emitter.initial_velocity_min *= theme::FIREWORK_SCALE;
        flash_emitter.initial_velocity_max *= theme::FIREWORK_SCALE;
        flash_emitter.size_start = theme::FIREWORK_SPARK_SIZE * 2.0;
        flash_emitter.size_end = 0.0;
        commands.set(flash, flash_emitter);
        commands.set(flash, Lifetime { remaining: 4.0 });

        let burst = commands.spawn(());
        let mut emitter =
            ParticleEmitter::firework_explosion(position, color, theme::FIREWORK_PARTICLES);
        emitter.initial_velocity_min *= theme::FIREWORK_SCALE;
        emitter.initial_velocity_max *= theme::FIREWORK_SCALE;
        emitter.size_start = theme::FIREWORK_SPARK_SIZE;
        emitter.size_end = 0.0;
        emitter.gravity *= theme::FIREWORK_SCALE * 2.0;
        commands.set(burst, emitter);
        commands.set(burst, Lifetime { remaining: 8.0 });
    }
}
