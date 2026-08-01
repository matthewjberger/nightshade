use crate::resources::Terrain;
use crate::tags::{Firefly, Scenery};
use crate::theme;
use nalgebra_glm::{Vec3, Vec4};
use nightshade::prelude::*;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};
use nightshade_scene::prelude::Lifetime;

pub fn spawn_fireflies(commands: &mut Commands, terrain: &Terrain, spots: &[(f32, f32)]) {
    for (x, z) in spots {
        let height =
            crate::queries::terrain::height_at(terrain, *x, *z).unwrap_or(theme::MIN_HEIGHT) as f32;
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
        commands.add_tag_type::<Scenery>(entity);
    }
}

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

pub fn hit_spark(commands: &mut Commands, position: Vec3, color: Vec3) {
    let entity = commands.spawn(());
    let mut emitter = ParticleEmitter::firework_explosion(position, color, theme::HIT_SPARK_COUNT);
    emitter.initial_velocity_min = 0.8;
    emitter.initial_velocity_max = 2.4;
    emitter.size_start = 0.07;
    emitter.size_end = 0.0;
    emitter.particle_lifetime_min = 0.3;
    emitter.particle_lifetime_max = 0.7;
    emitter.gravity = Vec3::new(0.0, -4.0, 0.0);
    commands.set(entity, emitter);
    commands.set(entity, Lifetime { remaining: 2.0 });
}
