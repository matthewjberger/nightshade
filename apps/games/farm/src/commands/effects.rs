use crate::tags::Firefly;

use crate::queries::terrain::height_at;
use crate::resources::{Fireworks, Shell, Terrain};
use crate::theme;
use nalgebra_glm::{Vec3, Vec4};
use nightshade::prelude::*;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};
use nightshade_scene::prelude::Lifetime;

pub fn sparkle(commands: &mut Commands, position: Vec3) {
    let entity = commands.spawn(());
    let mut emitter = ParticleEmitter::firework_glitter(position, theme::SPARKLE_COUNT);
    emitter.initial_velocity_min = 0.6;
    emitter.initial_velocity_max = 1.8;
    emitter.size_start = 0.05;
    emitter.size_end = 0.0;
    emitter.particle_lifetime_min = 0.5;
    emitter.particle_lifetime_max = 1.1;
    emitter.gravity = Vec3::new(0.0, -3.0, 0.0);
    commands.set(entity, emitter);
    commands.set(entity, Lifetime { remaining: 3.0 });
}

pub fn leaves(commands: &mut Commands, position: Vec3, color: Vec4) {
    let entity = commands.spawn(());
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Box {
            half_extents: Vec3::new(0.4, 0.5, 0.2),
        },
        position,
        direction: Vec3::new(0.0, 1.0, 0.0),
        spawn_rate: 0.0,
        burst_count: theme::LEAF_COUNT,
        one_shot: true,
        particle_lifetime_min: 0.8,
        particle_lifetime_max: 1.6,
        initial_velocity_min: 0.4,
        initial_velocity_max: 1.4,
        velocity_spread: std::f32::consts::PI,
        gravity: Vec3::new(0.0, -2.5, 0.0),
        drag: 1.5,
        size_start: 0.09,
        size_end: 0.04,
        color_gradient: ColorGradient {
            colors: vec![
                (0.0, color),
                (0.8, color),
                (1.0, Vec4::new(color.x, color.y, color.z, 0.0)),
            ],
        },
        emissive_strength: 0.0,
        turbulence_strength: 1.0,
        turbulence_frequency: 1.5,
        ..Default::default()
    };
    commands.set(entity, emitter);
    commands.set(entity, Lifetime { remaining: 3.0 });
}

pub fn spawn_smoke(commands: &mut Commands, position: Vec3) {
    let entity = commands.spawn(());
    let mut emitter = ParticleEmitter::smoke(position);
    emitter.spawn_rate = 6.0;
    emitter.size_start = 0.25;
    emitter.size_end = 0.9;
    emitter.initial_velocity_min = 0.4;
    emitter.initial_velocity_max = 0.7;
    emitter.particle_lifetime_min = 3.0;
    emitter.particle_lifetime_max = 5.0;
    emitter.velocity_spread = 0.25;
    emitter.turbulence_strength = 0.4;
    commands.set(entity, emitter);
}

pub fn spawn_fire(commands: &mut Commands, position: Vec3) {
    let entity = commands.spawn(());
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Box {
            half_extents: Vec3::new(0.18, 0.05, 0.18),
        },
        position,
        direction: Vec3::new(0.0, 1.0, 0.0),
        spawn_rate: 40.0,
        burst_count: 0,
        particle_lifetime_min: 0.4,
        particle_lifetime_max: 0.9,
        initial_velocity_min: 0.6,
        initial_velocity_max: 1.3,
        velocity_spread: 0.35,
        gravity: Vec3::new(0.0, 0.8, 0.0),
        drag: 0.8,
        size_start: 0.22,
        size_end: 0.02,
        color_gradient: ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 0.9, 0.5, 1.0)),
                (0.4, Vec4::new(1.0, 0.5, 0.15, 0.9)),
                (0.8, Vec4::new(0.6, 0.15, 0.05, 0.5)),
                (1.0, Vec4::new(0.2, 0.1, 0.1, 0.0)),
            ],
        },
        emissive_strength: 6.0,
        turbulence_strength: 0.6,
        turbulence_frequency: 2.0,
        ..Default::default()
    };
    commands.set(entity, emitter);
    spawn_smoke(commands, position + Vec3::new(0.0, 0.6, 0.0));
}

pub fn spawn_forge_sparks(commands: &mut Commands, position: Vec3) {
    let entity = commands.spawn(());
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Box {
            half_extents: Vec3::new(0.2, 0.1, 0.05),
        },
        position,
        direction: Vec3::new(0.0, 1.0, 0.3),
        spawn_rate: 6.0,
        burst_count: 0,
        particle_lifetime_min: 0.5,
        particle_lifetime_max: 1.2,
        initial_velocity_min: 0.8,
        initial_velocity_max: 1.8,
        velocity_spread: 0.7,
        gravity: Vec3::new(0.0, -3.0, 0.0),
        drag: 0.4,
        size_start: 0.05,
        size_end: 0.0,
        color_gradient: ColorGradient {
            colors: vec![
                (0.0, Vec4::new(1.0, 0.85, 0.4, 1.0)),
                (0.6, Vec4::new(1.0, 0.4, 0.1, 1.0)),
                (1.0, Vec4::new(0.4, 0.1, 0.0, 0.0)),
            ],
        },
        emissive_strength: 8.0,
        ..Default::default()
    };
    commands.set(entity, emitter);
}

pub fn spawn_fireflies(commands: &mut Commands, terrain: &Terrain, spots: &[(f32, f32)]) {
    for (x, z) in spots {
        let height = height_at(terrain, *x, *z).unwrap_or(theme::MIN_HEIGHT) as f32;
        let entity = commands.spawn(());
        let emitter = ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Box {
                half_extents: Vec3::new(2.0, 0.4, 1.5),
            },
            position: Vec3::new(*x, height + 0.8, *z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: theme::FIREFLY_RATE,
            burst_count: 0,
            particle_lifetime_min: 2.5,
            particle_lifetime_max: 5.0,
            initial_velocity_min: 0.1,
            initial_velocity_max: 0.4,
            velocity_spread: std::f32::consts::PI,
            gravity: Vec3::new(0.0, 0.05, 0.0),
            drag: 0.6,
            size_start: theme::FIREFLY_SIZE,
            size_end: 0.0,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, Vec4::new(1.0, 0.95, 0.4, 0.0)),
                    (0.2, Vec4::new(1.0, 0.95, 0.45, 1.0)),
                    (0.5, Vec4::new(0.85, 1.0, 0.35, 0.3)),
                    (0.7, Vec4::new(1.0, 0.95, 0.45, 1.0)),
                    (1.0, Vec4::new(0.6, 0.9, 0.3, 0.0)),
                ],
            },
            emissive_strength: 8.0,
            enabled: false,
            turbulence_strength: 1.2,
            turbulence_frequency: 0.8,
            ..Default::default()
        };
        commands.set(entity, emitter);
        commands.add_tag_type::<Firefly>(entity);
    }
}

pub fn spawn_rain(commands: &mut Commands, position: Vec3) -> Entity {
    let entity = commands.spawn(());
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Box {
            half_extents: theme::RAIN_REACH,
        },
        position,
        direction: Vec3::new(0.0, -1.0, 0.0),
        spawn_rate: theme::RAIN_RATE,
        burst_count: 0,
        particle_lifetime_min: 0.9,
        particle_lifetime_max: 1.1,
        initial_velocity_min: 11.0,
        initial_velocity_max: 13.0,
        velocity_spread: 0.03,
        gravity: Vec3::new(0.0, 0.0, 0.0),
        drag: 0.0,
        size_start: 0.09,
        size_end: 0.09,
        color_gradient: ColorGradient {
            colors: vec![
                (0.0, Vec4::new(0.75, 0.85, 1.0, 0.0)),
                (0.1, Vec4::new(0.75, 0.85, 1.0, 0.7)),
                (0.9, Vec4::new(0.75, 0.85, 1.0, 0.7)),
                (1.0, Vec4::new(0.75, 0.85, 1.0, 0.0)),
            ],
        },
        emissive_strength: 0.6,
        enabled: false,
        ..Default::default()
    };
    commands.set(entity, emitter);
    entity
}

pub fn spawn_wind_leaves(commands: &mut Commands, position: Vec3) -> Entity {
    let entity = commands.spawn(());
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Box {
            half_extents: Vec3::new(2.0, 2.5, 9.0),
        },
        position,
        direction: Vec3::new(1.0, -0.15, 0.0),
        spawn_rate: theme::LEAF_RATE,
        burst_count: 0,
        particle_lifetime_min: 3.0,
        particle_lifetime_max: 4.5,
        initial_velocity_min: theme::LEAF_SPEED * 0.8,
        initial_velocity_max: theme::LEAF_SPEED * 1.4,
        velocity_spread: 0.25,
        gravity: Vec3::new(0.0, -0.3, 0.0),
        drag: 0.0,
        size_start: 0.1,
        size_end: 0.08,
        color_gradient: ColorGradient {
            colors: vec![
                (0.0, Vec4::new(0.55, 0.7, 0.3, 0.0)),
                (0.15, Vec4::new(0.55, 0.7, 0.3, 0.9)),
                (0.5, Vec4::new(0.75, 0.55, 0.25, 0.9)),
                (1.0, Vec4::new(0.6, 0.4, 0.2, 0.0)),
            ],
        },
        emissive_strength: 0.0,
        enabled: false,
        turbulence_strength: 2.5,
        turbulence_frequency: 1.2,
        ..Default::default()
    };
    commands.set(entity, emitter);
    entity
}

pub fn launch_shell(commands: &mut Commands, fireworks: &mut Fireworks) {
    let rng = &mut fireworks.rng;
    let launch = Vec3::new(
        theme::FIREWORKS_LAUNCH.x + (rng.f32() - 0.5) * 6.0,
        theme::FIREWORKS_LAUNCH.y,
        theme::FIREWORKS_LAUNCH.z + (rng.f32() - 0.5) * 3.0,
    );
    let velocity = Vec3::new(
        (rng.f32() - 0.5) * 2.5,
        theme::FIREWORKS_SPEED_MIN
            + rng.f32() * (theme::FIREWORKS_SPEED_MAX - theme::FIREWORKS_SPEED_MIN),
        (rng.f32() - 0.5) * 1.5,
    );
    let color = theme::FIREWORK_COLORS
        [(rng.f32() * theme::FIREWORK_COLORS.len() as f32) as usize % theme::FIREWORK_COLORS.len()];
    let entity = commands.spawn(());
    let mut trail = ParticleEmitter::firework_shell(launch, velocity);
    trail.size_start = 0.08;
    trail.size_end = 0.0;
    commands.set(entity, trail);
    commands.set(entity, Lifetime { remaining: 8.0 });
    let fuse = theme::FIREWORKS_FUSE + rng.f32() * 0.4;
    let willow = rng.f32() > 0.7;
    fireworks.shells.push(Shell {
        entity,
        position: launch,
        velocity,
        fuse,
        color,
        willow,
    });
}

pub fn burst(commands: &mut Commands, position: Vec3, color: Vec3, willow: bool) {
    let flash = commands.spawn(());
    let mut flash_emitter = ParticleEmitter::flash_burst(position);
    flash_emitter.initial_velocity_min *= theme::FIREWORK_SCALE;
    flash_emitter.initial_velocity_max *= theme::FIREWORK_SCALE;
    flash_emitter.size_start = theme::FIREWORK_SPARK_SIZE * 2.0;
    flash_emitter.size_end = 0.0;
    commands.set(flash, flash_emitter);
    commands.set(flash, Lifetime { remaining: 4.0 });
    let entity = commands.spawn(());
    let mut emitter = if willow {
        ParticleEmitter::firework_willow(position, color, theme::FIREWORK_PARTICLES)
    } else {
        ParticleEmitter::firework_explosion(position, color, theme::FIREWORK_PARTICLES)
    };
    emitter.initial_velocity_min *= theme::FIREWORK_SCALE;
    emitter.initial_velocity_max *= theme::FIREWORK_SCALE;
    emitter.size_start = theme::FIREWORK_SPARK_SIZE;
    emitter.size_end = 0.0;
    emitter.gravity *= theme::FIREWORK_SCALE * 2.0;
    commands.set(entity, emitter);
    commands.set(entity, Lifetime { remaining: 8.0 });
}
