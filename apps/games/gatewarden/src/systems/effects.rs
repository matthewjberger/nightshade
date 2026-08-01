use crate::components::Burst;
use crate::data::effects::{BurstStyle, Puff};
use crate::theme::BURST_SECONDS;
use nightshade::prelude::*;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};
use nightshade_platform::prelude::*;

pub fn spawn_burst(commands: &mut Commands, origin: Vec3, style: BurstStyle) {
    let emitter_type = match style.puff {
        Puff::Sparks => EmitterType::Sparks,
        Puff::Glow => EmitterType::Firework,
        Puff::Smoke => EmitterType::Smoke,
    };
    let emitter = ParticleEmitter {
        emitter_type,
        position: origin,
        one_shot: true,
        burst_count: style.count,
        spawn_rate: 0.0,
        shape: EmitterShape::Sphere {
            radius: style.radius,
        },
        initial_velocity_min: style.speed_min,
        initial_velocity_max: style.speed_max,
        velocity_spread: style.spread,
        gravity: Vec3::new(0.0, style.rise, 0.0),
        drag: style.drag,
        particle_lifetime_min: style.life_min,
        particle_lifetime_max: style.life_max,
        size_start: style.size_start,
        size_end: style.size_end,
        emissive_strength: style.emissive,
        color_gradient: ColorGradient {
            colors: vec![
                (0.0, style.colors[0]),
                (0.5, style.colors[1]),
                (1.0, style.colors[2]),
            ],
        },
        ..ParticleEmitter::default()
    };
    commands.spawn((emitter, Burst::default()));
}

pub fn update(time: Res<Time>, mut commands: Commands, mut bursts: Query<(&mut Burst,)>) {
    let mut expired = Vec::new();
    bursts.for_each_mut(|entity, (burst,)| {
        burst.age += time.delta_time;
        if burst.age > BURST_SECONDS {
            expired.push(entity);
        }
    });
    for entity in expired {
        commands.despawn(entity);
    }
}
