use crate::resources::{Burst, BurstKind, Bursts};
use nightshade::prelude::*;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};
use nightshade_scene::prelude::Lifetime;

pub fn burst(bursts: &mut Bursts, kind: BurstKind, position: Vec3) {
    bursts.pending.push(Burst { kind, position });
}

fn gradient(bright: Vec4, warm: Vec4, dark: Vec4) -> ColorGradient {
    ColorGradient {
        colors: vec![
            (0.0, bright),
            (0.3, warm),
            (0.7, Vec4::new(dark.x, dark.y, dark.z, dark.w * 0.6)),
            (1.0, Vec4::new(dark.x, dark.y, dark.z, 0.0)),
        ],
    }
}

fn spray(
    position: Vec3,
    count: u32,
    speed: (f32, f32),
    life: (f32, f32),
    size: f32,
    gravity: f32,
    colors: ColorGradient,
    emissive: f32,
) -> ParticleEmitter {
    ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Sphere { radius: 0.2 },
        position,
        direction: Vec3::new(0.0, 1.0, 0.0),
        spawn_rate: 0.0,
        burst_count: count,
        one_shot: true,
        particle_lifetime_min: life.0,
        particle_lifetime_max: life.1,
        initial_velocity_min: speed.0,
        initial_velocity_max: speed.1,
        velocity_spread: std::f32::consts::PI,
        gravity: Vec3::new(0.0, gravity, 0.0),
        drag: 0.6,
        size_start: size,
        size_end: 0.0,
        color_gradient: colors,
        emissive_strength: emissive,
        turbulence_strength: 0.3,
        turbulence_frequency: 2.0,
        ..Default::default()
    }
}

fn smoke(position: Vec3, count: u32, size: f32) -> ParticleEmitter {
    ParticleEmitter {
        emitter_type: EmitterType::Smoke,
        shape: EmitterShape::Sphere { radius: 0.3 },
        position,
        direction: Vec3::new(0.0, 1.0, 0.0),
        spawn_rate: 0.0,
        burst_count: count,
        one_shot: true,
        particle_lifetime_min: 0.8,
        particle_lifetime_max: 1.5,
        initial_velocity_min: 0.4,
        initial_velocity_max: 1.2,
        velocity_spread: std::f32::consts::PI,
        gravity: Vec3::new(0.0, 0.6, 0.0),
        drag: 1.2,
        size_start: size,
        size_end: size * 2.5,
        color_gradient: ColorGradient::smoke(),
        emissive_strength: 0.0,
        turbulence_strength: 0.5,
        turbulence_frequency: 1.0,
        ..Default::default()
    }
}

fn pillar(position: Vec3, count: u32, colors: ColorGradient) -> ParticleEmitter {
    ParticleEmitter {
        emitter_type: EmitterType::Firework,
        shape: EmitterShape::Sphere { radius: 0.7 },
        position,
        direction: Vec3::new(0.0, 1.0, 0.0),
        spawn_rate: 0.0,
        burst_count: count,
        one_shot: true,
        particle_lifetime_min: 1.0,
        particle_lifetime_max: 2.0,
        initial_velocity_min: 2.5,
        initial_velocity_max: 6.5,
        velocity_spread: 0.45,
        gravity: Vec3::new(0.0, -1.2, 0.0),
        drag: 0.5,
        size_start: 0.16,
        size_end: 0.0,
        color_gradient: colors,
        emissive_strength: 8.0,
        turbulence_strength: 0.6,
        turbulence_frequency: 1.5,
        ..Default::default()
    }
}

pub fn fire_trail(position: Vec3) -> ParticleEmitter {
    ParticleEmitter {
        emitter_type: EmitterType::Fire,
        shape: EmitterShape::Sphere { radius: 0.12 },
        position,
        direction: Vec3::new(0.0, 1.0, 0.0),
        spawn_rate: 110.0,
        burst_count: 0,
        particle_lifetime_min: 0.18,
        particle_lifetime_max: 0.45,
        initial_velocity_min: 0.2,
        initial_velocity_max: 0.8,
        velocity_spread: std::f32::consts::PI,
        gravity: Vec3::new(0.0, 1.5, 0.0),
        drag: 0.5,
        size_start: 0.26,
        size_end: 0.04,
        color_gradient: ColorGradient::fire(),
        emissive_strength: 6.0,
        turbulence_strength: 0.4,
        turbulence_frequency: 3.0,
        ..Default::default()
    }
}

pub fn frost_trail(position: Vec3) -> ParticleEmitter {
    let mut emitter = fire_trail(position);
    emitter.emitter_type = EmitterType::Sparks;
    emitter.gravity = Vec3::new(0.0, -0.5, 0.0);
    emitter.size_start = 0.18;
    emitter.color_gradient = gradient(
        Vec4::new(0.9, 0.98, 1.0, 1.0),
        Vec4::new(0.55, 0.8, 1.0, 0.9),
        Vec4::new(0.3, 0.5, 0.9, 0.6),
    );
    emitter
}

pub fn spawn_burst(commands: &mut Commands, burst: Burst) {
    let position = burst.position;
    let gold = gradient(
        Vec4::new(1.0, 1.0, 0.85, 1.0),
        Vec4::new(1.0, 0.85, 0.35, 1.0),
        Vec4::new(0.8, 0.5, 0.1, 0.7),
    );
    let sparks = gradient(
        Vec4::new(1.0, 1.0, 0.8, 1.0),
        Vec4::new(1.0, 0.8, 0.3, 1.0),
        Vec4::new(1.0, 0.4, 0.1, 0.7),
    );
    let embers = gradient(
        Vec4::new(1.0, 0.95, 0.7, 1.0),
        Vec4::new(1.0, 0.45, 0.1, 1.0),
        Vec4::new(0.7, 0.15, 0.02, 0.8),
    );
    let frost = gradient(
        Vec4::new(0.95, 1.0, 1.0, 1.0),
        Vec4::new(0.6, 0.85, 1.0, 0.95),
        Vec4::new(0.3, 0.5, 0.9, 0.6),
    );
    let green = gradient(
        Vec4::new(0.85, 1.0, 0.8, 1.0),
        Vec4::new(0.45, 1.0, 0.5, 1.0),
        Vec4::new(0.2, 0.7, 0.3, 0.6),
    );
    let blood = gradient(
        Vec4::new(0.9, 0.3, 0.25, 1.0),
        Vec4::new(0.6, 0.1, 0.1, 1.0),
        Vec4::new(0.3, 0.05, 0.05, 0.7),
    );
    let leaves = gradient(
        Vec4::new(0.8, 0.95, 0.4, 1.0),
        Vec4::new(0.55, 0.8, 0.25, 1.0),
        Vec4::new(0.3, 0.5, 0.15, 0.7),
    );
    let dust = gradient(
        Vec4::new(0.7, 0.62, 0.5, 0.55),
        Vec4::new(0.6, 0.55, 0.45, 0.4),
        Vec4::new(0.5, 0.45, 0.4, 0.2),
    );
    let violet = gradient(
        Vec4::new(1.0, 0.9, 1.0, 1.0),
        Vec4::new(0.75, 0.45, 1.0, 1.0),
        Vec4::new(0.4, 0.2, 0.7, 0.7),
    );

    let mut emitters = Vec::new();
    match burst.kind {
        BurstKind::Hit => emitters.push(spray(
            position,
            16,
            (2.0, 5.0),
            (0.25, 0.55),
            0.09,
            -7.0,
            sparks,
            4.0,
        )),
        BurstKind::Crit => {
            emitters.push(spray(
                position,
                40,
                (3.5, 8.5),
                (0.3, 0.7),
                0.14,
                -7.0,
                embers,
                6.0,
            ));
            emitters.push(spray(
                position,
                10,
                (0.5, 1.5),
                (0.2, 0.35),
                0.5,
                0.0,
                ColorGradient::flash_burst(),
                8.0,
            ));
        }
        BurstKind::Frost | BurstKind::FrostHit => {
            emitters.push(spray(
                position,
                26,
                (1.0, 3.5),
                (0.5, 1.0),
                0.11,
                -1.5,
                frost,
                5.0,
            ));
        }
        BurstKind::Heal => {
            let mut rising = spray(position, 30, (0.8, 2.2), (0.7, 1.3), 0.11, 1.8, green, 5.0);
            rising.shape = EmitterShape::Sphere { radius: 0.5 };
            emitters.push(rising);
        }
        BurstKind::Kill => {
            emitters.push(spray(
                position,
                30,
                (2.0, 6.0),
                (0.4, 0.9),
                0.12,
                -8.0,
                blood,
                0.5,
            ));
            emitters.push(smoke(position, 8, 0.35));
        }
        BurstKind::EliteKill => {
            emitters.push(spray(
                position,
                70,
                (3.0, 9.0),
                (0.5, 1.2),
                0.16,
                -6.0,
                violet,
                7.0,
            ));
            emitters.push(smoke(position, 18, 0.6));
            emitters.push(pillar(position, 60, gold.clone()));
        }
        BurstKind::Coins => emitters.push(spray(
            position + Vec3::new(0.0, 0.6, 0.0),
            14,
            (1.5, 3.5),
            (0.5, 0.9),
            0.09,
            -5.0,
            gold,
            6.0,
        )),
        BurstKind::LevelUp => {
            emitters.push(pillar(position, 120, gold.clone()));
            let mut ring = spray(
                position + Vec3::new(0.0, 0.3, 0.0),
                60,
                (3.0, 5.0),
                (0.6, 1.2),
                0.12,
                -2.0,
                ColorGradient::firework_glitter(),
                8.0,
            );
            ring.shape = EmitterShape::Sphere { radius: 0.8 };
            emitters.push(ring);
        }
        BurstKind::QuestDone => emitters.push(pillar(position, 50, gold)),
        BurstKind::Gather => emitters.push(spray(
            position,
            16,
            (0.8, 2.0),
            (0.5, 0.9),
            0.09,
            -3.0,
            leaves,
            0.5,
        )),
        BurstKind::Dust => {
            let mut puff = spray(position, 5, (0.3, 0.9), (0.3, 0.6), 0.16, 0.4, dust, 0.0);
            puff.shape = EmitterShape::Sphere { radius: 0.3 };
            puff.drag = 2.0;
            emitters.push(puff);
        }
        BurstKind::Puff => emitters.push(smoke(position, 12, 0.4)),
        BurstKind::Cast => emitters.push(spray(
            position,
            14,
            (0.5, 1.5),
            (0.2, 0.4),
            0.2,
            1.0,
            embers,
            6.0,
        )),
        BurstKind::FrostCast => emitters.push(spray(
            position,
            14,
            (0.5, 1.5),
            (0.2, 0.4),
            0.18,
            -0.5,
            frost,
            5.0,
        )),
        BurstKind::Fire => {
            emitters.push(spray(
                position,
                36,
                (2.0, 6.0),
                (0.3, 0.7),
                0.2,
                -3.0,
                embers,
                6.0,
            ));
            emitters.push(smoke(position, 6, 0.3));
        }
    }

    for emitter in emitters {
        let entity = commands.spawn(());
        commands.set(entity, emitter);
        commands.set(entity, Lifetime { remaining: 4.0 });
    }
}
