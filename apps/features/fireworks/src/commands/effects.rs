use crate::components::Effect;
use crate::data::EffectKind;
use nightshade::prelude::*;
use nightshade_particles::contract::*;
use nightshade_text::contract::resources::TextState;
use nightshade_text::prelude::*;

pub fn spawn_fire_effect(commands: &mut Commands<'_>, position: Vec3) {
    let entity = commands.spawn(());
    commands.set(entity, ParticleEmitter::fire(position));
    commands.set(entity, Effect(EffectKind::Fire));
}

pub fn spawn_smoke_effect(commands: &mut Commands<'_>, position: Vec3) {
    let entity = commands.spawn(());
    commands.set(entity, ParticleEmitter::smoke(position));
    commands.set(entity, Effect(EffectKind::Smoke));
}

fn spawn_sprite_effect(commands: &mut Commands<'_>, emitter: ParticleEmitter) {
    let entity = commands.spawn(());
    commands.set(entity, emitter);
    commands.set(entity, Effect(EffectKind::Sprite));
}

fn spawn_sprite_label(
    text_state: &mut TextState,
    commands: &mut Commands<'_>,
    position: Vec3,
    text: &str,
) {
    spawn_3d_billboard_text_with_properties(
        text_state,
        commands,
        text,
        position,
        TextProperties {
            font_size: 24.0,
            color: vec4(1.0, 1.0, 1.0, 0.9),
            alignment: TextAlignment::Center,
            vertical_alignment: VerticalAlignment::Middle,
            outline_width: 1.5,
            outline_color: vec4(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        },
    );
}

pub fn spawn_all_sprite_effects(text_state: &mut TextState, commands: &mut Commands<'_>) {
    let ground_z = 25.0;
    let spacing = 15.0;
    let start_x = -4.0 * spacing;
    let label_height = 12.0;
    let column = |index: f32| start_x + index * spacing;

    spawn_sprite_label(
        text_state,
        commands,
        Vec3::new(column(0.0), label_height, ground_z),
        "star_04",
    );
    spawn_sprite_effect(
        commands,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Point,
            position: Vec3::new(column(0.0), 0.0, ground_z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 15.0,
            particle_lifetime_min: 0.8,
            particle_lifetime_max: 1.8,
            initial_velocity_min: 4.0,
            initial_velocity_max: 10.0,
            velocity_spread: 0.5,
            gravity: Vec3::new(0.0, -6.0, 0.0),
            drag: 0.1,
            size_start: 3.0,
            size_end: 0.5,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(1.0, 1.0, 1.0, 1.0)),
                    (0.2, vec4(1.0, 0.95, 0.4, 1.0)),
                    (0.6, vec4(1.0, 0.7, 0.1, 0.9)),
                    (1.0, vec4(0.8, 0.3, 0.0, 0.0)),
                ],
            },
            emissive_strength: 12.0,
            turbulence_strength: 0.0,
            turbulence_frequency: 0.0,
            texture_index: 9,
            ..Default::default()
        },
    );

    spawn_sprite_label(
        text_state,
        commands,
        Vec3::new(column(1.0), label_height, ground_z),
        "symbol_01",
    );
    spawn_sprite_effect(
        commands,
        ParticleEmitter {
            emitter_type: EmitterType::Fire,
            shape: EmitterShape::Sphere { radius: 0.5 },
            position: Vec3::new(column(1.0), 0.5, ground_z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 8.0,
            particle_lifetime_min: 2.0,
            particle_lifetime_max: 4.0,
            initial_velocity_min: 0.5,
            initial_velocity_max: 2.0,
            velocity_spread: 0.8,
            gravity: Vec3::new(0.0, 1.5, 0.0),
            drag: 0.3,
            size_start: 2.5,
            size_end: 5.0,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(1.0, 0.3, 0.5, 0.9)),
                    (0.3, vec4(1.0, 0.2, 0.4, 0.8)),
                    (0.7, vec4(0.9, 0.1, 0.3, 0.5)),
                    (1.0, vec4(0.6, 0.0, 0.2, 0.0)),
                ],
            },
            emissive_strength: 6.0,
            turbulence_strength: 0.8,
            turbulence_frequency: 0.8,
            texture_index: 21,
            ..Default::default()
        },
    );

    spawn_sprite_label(
        text_state,
        commands,
        Vec3::new(column(2.0), label_height, ground_z),
        "magic_01",
    );
    spawn_sprite_effect(
        commands,
        ParticleEmitter {
            emitter_type: EmitterType::Fire,
            shape: EmitterShape::Sphere { radius: 1.5 },
            position: Vec3::new(column(2.0), 2.0, ground_z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 6.0,
            particle_lifetime_min: 2.5,
            particle_lifetime_max: 5.0,
            initial_velocity_min: 0.3,
            initial_velocity_max: 1.5,
            velocity_spread: std::f32::consts::PI,
            gravity: Vec3::new(0.0, 0.3, 0.0),
            drag: 0.2,
            size_start: 3.0,
            size_end: 6.0,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(0.5, 0.2, 1.0, 0.8)),
                    (0.3, vec4(0.7, 0.3, 1.0, 0.7)),
                    (0.6, vec4(0.9, 0.5, 1.0, 0.4)),
                    (1.0, vec4(1.0, 0.8, 1.0, 0.0)),
                ],
            },
            emissive_strength: 5.0,
            turbulence_strength: 1.0,
            turbulence_frequency: 0.5,
            texture_index: 12,
            ..Default::default()
        },
    );

    spawn_sprite_label(
        text_state,
        commands,
        Vec3::new(column(3.0), label_height, ground_z),
        "star_06",
    );
    spawn_sprite_effect(
        commands,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Point,
            position: Vec3::new(column(3.0), 0.0, ground_z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 12.0,
            particle_lifetime_min: 1.0,
            particle_lifetime_max: 2.5,
            initial_velocity_min: 3.0,
            initial_velocity_max: 8.0,
            velocity_spread: 0.7,
            gravity: Vec3::new(0.0, -4.0, 0.0),
            drag: 0.15,
            size_start: 3.5,
            size_end: 1.0,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(1.0, 1.0, 1.0, 1.0)),
                    (0.15, vec4(0.6, 0.8, 1.0, 1.0)),
                    (0.5, vec4(0.3, 0.5, 1.0, 0.8)),
                    (1.0, vec4(0.1, 0.2, 0.8, 0.0)),
                ],
            },
            emissive_strength: 10.0,
            turbulence_strength: 0.0,
            turbulence_frequency: 0.0,
            texture_index: 10,
            ..Default::default()
        },
    );

    spawn_sprite_label(
        text_state,
        commands,
        Vec3::new(column(4.0), label_height, ground_z),
        "circle_05",
    );
    spawn_sprite_effect(
        commands,
        ParticleEmitter {
            emitter_type: EmitterType::Smoke,
            shape: EmitterShape::Sphere { radius: 1.0 },
            position: Vec3::new(column(4.0), 1.0, ground_z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 8.0,
            particle_lifetime_min: 3.0,
            particle_lifetime_max: 6.0,
            initial_velocity_min: 0.2,
            initial_velocity_max: 1.0,
            velocity_spread: std::f32::consts::PI,
            gravity: Vec3::new(0.0, 0.5, 0.0),
            drag: 0.1,
            size_start: 2.0,
            size_end: 5.0,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(0.2, 0.9, 1.0, 0.7)),
                    (0.3, vec4(0.3, 0.8, 1.0, 0.6)),
                    (0.6, vec4(0.1, 0.6, 0.9, 0.4)),
                    (1.0, vec4(0.05, 0.3, 0.7, 0.0)),
                ],
            },
            emissive_strength: 3.0,
            turbulence_strength: 0.5,
            turbulence_frequency: 0.3,
            texture_index: 15,
            ..Default::default()
        },
    );

    spawn_sprite_label(
        text_state,
        commands,
        Vec3::new(column(5.0), label_height, ground_z),
        "scorch_01",
    );
    spawn_sprite_effect(
        commands,
        ParticleEmitter {
            emitter_type: EmitterType::Fire,
            shape: EmitterShape::Sphere { radius: 0.5 },
            position: Vec3::new(column(5.0), 0.5, ground_z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 10.0,
            particle_lifetime_min: 0.8,
            particle_lifetime_max: 2.0,
            initial_velocity_min: 1.0,
            initial_velocity_max: 4.0,
            velocity_spread: std::f32::consts::PI,
            gravity: Vec3::new(0.0, -2.0, 0.0),
            drag: 0.2,
            size_start: 4.0,
            size_end: 1.5,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(1.0, 1.0, 0.9, 1.0)),
                    (0.2, vec4(1.0, 0.8, 0.3, 0.9)),
                    (0.5, vec4(1.0, 0.5, 0.1, 0.7)),
                    (1.0, vec4(0.6, 0.15, 0.0, 0.0)),
                ],
            },
            emissive_strength: 8.0,
            turbulence_strength: 0.3,
            turbulence_frequency: 1.0,
            texture_index: 19,
            ..Default::default()
        },
    );

    spawn_sprite_label(
        text_state,
        commands,
        Vec3::new(column(6.0), label_height, ground_z),
        "twirl_01",
    );
    spawn_sprite_effect(
        commands,
        ParticleEmitter {
            emitter_type: EmitterType::Fire,
            shape: EmitterShape::Sphere { radius: 0.5 },
            position: Vec3::new(column(6.0), 0.5, ground_z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 10.0,
            particle_lifetime_min: 2.0,
            particle_lifetime_max: 4.0,
            initial_velocity_min: 0.5,
            initial_velocity_max: 2.5,
            velocity_spread: std::f32::consts::PI,
            gravity: Vec3::new(0.0, 1.0, 0.0),
            drag: 0.25,
            size_start: 2.5,
            size_end: 4.5,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(0.1, 1.0, 0.6, 0.7)),
                    (0.3, vec4(0.2, 0.9, 0.8, 0.6)),
                    (0.6, vec4(0.3, 0.8, 1.0, 0.4)),
                    (1.0, vec4(0.5, 1.0, 0.9, 0.0)),
                ],
            },
            emissive_strength: 4.0,
            turbulence_strength: 1.5,
            turbulence_frequency: 0.8,
            texture_index: 11,
            ..Default::default()
        },
    );

    spawn_sprite_label(
        text_state,
        commands,
        Vec3::new(column(7.0), label_height, ground_z),
        "muzzle_03",
    );
    spawn_sprite_effect(
        commands,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Cone {
                angle: 0.3,
                height: 0.1,
            },
            position: Vec3::new(column(7.0), 0.0, ground_z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 12.0,
            particle_lifetime_min: 0.5,
            particle_lifetime_max: 1.5,
            initial_velocity_min: 5.0,
            initial_velocity_max: 12.0,
            velocity_spread: 0.4,
            gravity: Vec3::new(0.0, -8.0, 0.0),
            drag: 0.05,
            size_start: 3.5,
            size_end: 1.0,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(1.0, 1.0, 1.0, 1.0)),
                    (0.15, vec4(1.0, 0.8, 0.2, 1.0)),
                    (0.4, vec4(1.0, 0.5, 0.05, 0.9)),
                    (0.7, vec4(1.0, 0.2, 0.0, 0.6)),
                    (1.0, vec4(0.5, 0.05, 0.0, 0.0)),
                ],
            },
            emissive_strength: 10.0,
            turbulence_strength: 0.0,
            turbulence_frequency: 0.0,
            texture_index: 23,
            ..Default::default()
        },
    );

    spawn_sprite_label(
        text_state,
        commands,
        Vec3::new(column(8.0), label_height, ground_z),
        "window_01",
    );
    spawn_sprite_effect(
        commands,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.3 },
            position: Vec3::new(column(8.0), 0.0, ground_z),
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 6.0,
            particle_lifetime_min: 2.0,
            particle_lifetime_max: 4.0,
            initial_velocity_min: 2.0,
            initial_velocity_max: 5.0,
            velocity_spread: 0.6,
            gravity: Vec3::new(0.0, -3.0, 0.0),
            drag: 0.15,
            size_start: 4.0,
            size_end: 2.0,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, vec4(1.0, 1.0, 1.0, 0.9)),
                    (0.2, vec4(1.0, 0.9, 0.6, 0.8)),
                    (0.5, vec4(0.9, 0.8, 0.4, 0.6)),
                    (1.0, vec4(0.6, 0.5, 0.2, 0.0)),
                ],
            },
            emissive_strength: 5.0,
            turbulence_strength: 0.5,
            turbulence_frequency: 0.5,
            texture_index: 24,
            ..Default::default()
        },
    );
}
