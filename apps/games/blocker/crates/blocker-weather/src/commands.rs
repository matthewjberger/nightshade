use crate::theme;
use nalgebra_glm::{Vec3, vec4};
use nightshade::ecs::Entity;
use nightshade::prelude::Commands;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};

pub fn spawn_rainfall(commands: &mut Commands) -> Entity {
    let entity = commands.spawn(());
    commands.set(
        entity,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Box {
                half_extents: theme::CELL_HALF,
            },
            position: Vec3::zeros(),
            direction: Vec3::new(0.0, -1.0, 0.0),
            spawn_rate: theme::RAIN_RATE,
            burst_count: 0,
            particle_lifetime_min: 0.8,
            particle_lifetime_max: 0.8,
            initial_velocity_min: theme::RAIN_SPEED,
            initial_velocity_max: theme::RAIN_SPEED,
            velocity_spread: 0.03,
            gravity: Vec3::zeros(),
            drag: 0.0,
            size_start: 0.06,
            size_end: 0.06,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(0.72, 0.82, 1.0, 0.0)),
                    (0.1, vec4(0.72, 0.82, 1.0, 0.65)),
                    (0.9, vec4(0.72, 0.82, 1.0, 0.65)),
                    (1.0, vec4(0.72, 0.82, 1.0, 0.0)),
                ],
            },
            emissive_strength: 0.5,
            enabled: false,
            ..Default::default()
        },
    );
    entity
}

pub fn spawn_splashes(commands: &mut Commands) -> Entity {
    let entity = commands.spawn(());
    commands.set(
        entity,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Box {
                half_extents: Vec3::new(theme::CELL_HALF.x, 0.0, theme::CELL_HALF.z),
            },
            position: Vec3::zeros(),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: theme::SPLASH_RATE,
            burst_count: 0,
            particle_lifetime_min: 0.18,
            particle_lifetime_max: 0.3,
            initial_velocity_min: 1.2,
            initial_velocity_max: 2.2,
            velocity_spread: 0.7,
            gravity: Vec3::new(0.0, -9.8, 0.0),
            drag: 0.0,
            size_start: 0.05,
            size_end: 0.02,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(0.78, 0.86, 1.0, 0.7)),
                    (1.0, vec4(0.78, 0.86, 1.0, 0.0)),
                ],
            },
            emissive_strength: 0.4,
            enabled: false,
            ..Default::default()
        },
    );
    entity
}

pub fn spawn_snowfall(commands: &mut Commands) -> Entity {
    let entity = commands.spawn(());
    commands.set(
        entity,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Box {
                half_extents: theme::CELL_HALF,
            },
            position: Vec3::zeros(),
            direction: Vec3::new(0.0, -1.0, 0.0),
            spawn_rate: theme::SNOW_RATE,
            burst_count: 0,
            particle_lifetime_min: 6.0,
            particle_lifetime_max: 6.0,
            initial_velocity_min: theme::SNOW_SPEED,
            initial_velocity_max: theme::SNOW_SPEED,
            velocity_spread: 0.2,
            gravity: Vec3::zeros(),
            drag: 0.0,
            size_start: 0.11,
            size_end: 0.11,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(1.0, 1.0, 1.0, 0.0)),
                    (0.15, vec4(1.0, 1.0, 1.0, 0.9)),
                    (0.85, vec4(1.0, 1.0, 1.0, 0.9)),
                    (1.0, vec4(1.0, 1.0, 1.0, 0.0)),
                ],
            },
            emissive_strength: 0.3,
            enabled: false,
            ..Default::default()
        },
    );
    entity
}
