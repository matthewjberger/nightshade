use crate::theme;
use nalgebra_glm::{Vec3, Vec4};
use nightshade::prelude::*;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};
use nightshade_scene::prelude::Lifetime;

fn gradient(color: [f32; 3], tail: f32) -> ColorGradient {
    ColorGradient {
        colors: vec![
            (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
            (0.25, Vec4::new(color[0], color[1], color[2], 1.0)),
            (tail, Vec4::new(color[0], color[1], color[2], 0.8)),
            (
                1.0,
                Vec4::new(color[0] * 0.4, color[1] * 0.4, color[2] * 0.4, 0.0),
            ),
        ],
    }
}

fn burst(
    commands: &mut Commands,
    at: Vec3,
    color: [f32; 3],
    count: u32,
    speed: (f32, f32),
    size: (f32, f32),
    life: (f32, f32),
    fall: f32,
) {
    let entity = commands.spawn(());
    commands.set(
        entity,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.12 },
            position: at,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: count,
            one_shot: true,
            particle_lifetime_min: life.0,
            particle_lifetime_max: life.1,
            initial_velocity_min: speed.0,
            initial_velocity_max: speed.1,
            velocity_spread: std::f32::consts::PI,
            gravity: Vec3::new(0.0, -fall, 0.0),
            drag: 1.1,
            size_start: size.0,
            size_end: size.1,
            color_gradient: gradient(color, 0.7),
            emissive_strength: 3.0,
            turbulence_strength: 0.4,
            turbulence_frequency: 1.4,
            ..Default::default()
        },
    );
    commands.set(entity, Lifetime { remaining: 2.6 });
}

pub fn crystal_breaks(commands: &mut Commands, at: Vec3) {
    burst(
        commands,
        at,
        theme::SHARD_SPARK,
        theme::BURST_COUNT,
        (1.6, 4.4),
        (0.13, 0.02),
        (0.35, 0.85),
        5.0,
    );
    burst(
        commands,
        at,
        theme::COIN_SPARK,
        8,
        (0.8, 2.0),
        (0.07, 0.01),
        (0.5, 0.9),
        2.0,
    );
}

pub fn tower_fires(commands: &mut Commands, at: Vec3) {
    burst(
        commands,
        at,
        theme::COIN_SPARK,
        theme::MUZZLE_COUNT,
        (0.7, 1.9),
        (0.07, 0.0),
        (0.14, 0.3),
        3.0,
    );
}

pub fn stone_settles(commands: &mut Commands, at: Vec3) {
    burst(
        commands,
        at,
        [0.78, 0.74, 0.66],
        theme::DUST_COUNT,
        (0.9, 2.4),
        (0.11, 0.0),
        (0.35, 0.7),
        4.0,
    );
}

pub fn keep_is_struck(commands: &mut Commands, at: Vec3) {
    burst(
        commands,
        at,
        theme::BREACH_SPARK,
        theme::BREACH_COUNT,
        (2.0, 5.2),
        (0.16, 0.02),
        (0.4, 1.0),
        4.0,
    );
}

pub fn splash_lands(commands: &mut Commands, at: Vec3) {
    burst(
        commands,
        at,
        theme::SPLASH_SPARK,
        theme::SPLASH_COUNT,
        (2.6, 5.0),
        (0.1, 0.0),
        (0.22, 0.4),
        1.2,
    );
}

pub fn wave_is_cleared(commands: &mut Commands, at: Vec3) {
    burst(
        commands,
        at,
        theme::COIN_SPARK,
        theme::CLEARED_COUNT,
        (1.4, 3.8),
        (0.13, 0.0),
        (0.6, 1.3),
        2.2,
    );
}

pub fn tower_rises(commands: &mut Commands, at: Vec3) {
    burst(
        commands,
        at,
        theme::COIN_SPARK,
        theme::DUST_COUNT,
        (1.2, 3.0),
        (0.1, 0.0),
        (0.4, 0.9),
        3.0,
    );
}
