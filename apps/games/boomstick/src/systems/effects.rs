use crate::data;
use crate::tags::LevelPiece;
use nightshade::prelude::*;
use nightshade_graphics::prelude::spawn_light_entity;
use nightshade_particles::contract::ParticleTextures;
use nightshade_particles::contract::components::ParticleEmitter;
use nightshade_particles::contract::data::{
    ColorGradient, EmitterShape, EmitterType, ParticleTextureUpload,
};
use nightshade_render::contract::{Light, LightType};
use nightshade_render::data::decoded_image::decode_to_rgba8;
use nightshade_scene::contract::Lifetime;
use nightshade_scene::prelude::*;

const MUZZLE: &[u8] = include_bytes!("../../assets/textures/muzzle.png");
const SPARK: &[u8] = include_bytes!("../../assets/textures/spark.png");
const SMOKE: &[u8] = include_bytes!("../../assets/textures/smoke.png");
const SCORCH: &[u8] = include_bytes!("../../assets/textures/scorch.png");

pub fn load(textures: &mut ParticleTextures) {
    for (slot, bytes) in [
        (data::MUZZLE_SLOT, MUZZLE),
        (data::SPARK_SLOT, SPARK),
        (data::SMOKE_SLOT, SMOKE),
        (data::SCORCH_SLOT, SCORCH),
    ] {
        let Ok(picture) = decode_to_rgba8(bytes) else {
            continue;
        };
        let (width, height) = (picture.width, picture.height);
        textures.pending.push(ParticleTextureUpload {
            slot,
            rgba_data: picture.rgba,
            width,
            height,
        });
    }
}

pub fn muzzle(commands: &mut Commands, at: Vec3, direction: Vec3) {
    burst(
        commands,
        at,
        ParticleEmitter {
            emitter_type: EmitterType::Fire,
            shape: EmitterShape::Cone {
                angle: 0.35,
                height: 0.4,
            },
            position: at,
            direction,
            burst_count: 14,
            one_shot: true,
            particle_lifetime_min: 0.04,
            particle_lifetime_max: 0.11,
            initial_velocity_min: 3.0,
            initial_velocity_max: 9.0,
            velocity_spread: 0.35,
            gravity: Vec3::zeros(),
            drag: 4.0,
            size_start: 0.32,
            size_end: 0.02,
            emissive_strength: 6.0,
            texture_index: data::MUZZLE_SLOT,
            ..Default::default()
        },
        0.3,
    );
}

pub fn sparks(commands: &mut Commands, at: Vec3, normal: Vec3) {
    burst(
        commands,
        at,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Cone {
                angle: 0.4,
                height: 0.4,
            },
            position: at,
            direction: normal,
            burst_count: 10,
            one_shot: true,
            particle_lifetime_min: 0.12,
            particle_lifetime_max: 0.4,
            initial_velocity_min: 2.0,
            initial_velocity_max: 7.0,
            velocity_spread: 0.8,
            gravity: Vec3::new(0.0, -16.0, 0.0),
            drag: 1.2,
            size_start: 0.08,
            size_end: 0.0,
            emissive_strength: 4.0,
            texture_index: data::SPARK_SLOT,
            ..Default::default()
        },
        0.7,
    );
}

pub fn blood(commands: &mut Commands, at: Vec3) {
    burst(
        commands,
        at,
        ParticleEmitter {
            emitter_type: EmitterType::Smoke,
            shape: EmitterShape::Sphere { radius: 0.3 },
            position: at,
            direction: Vec3::y(),
            burst_count: 18,
            one_shot: true,
            particle_lifetime_min: 0.25,
            particle_lifetime_max: 0.7,
            initial_velocity_min: 1.4,
            initial_velocity_max: 5.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -14.0, 0.0),
            drag: 1.6,
            size_start: 0.22,
            size_end: 0.04,
            emissive_strength: 0.8,
            color_gradient: fade(
                Vec4::new(0.62, 0.06, 0.08, 1.0),
                Vec4::new(0.24, 0.02, 0.03, 0.0),
            ),
            texture_index: data::SCORCH_SLOT,
            ..Default::default()
        },
        1.0,
    );
}

pub fn gibs(commands: &mut Commands, at: Vec3) {
    blood(commands, at);
    burst(
        commands,
        at,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere { radius: 0.4 },
            position: at,
            direction: Vec3::y(),
            burst_count: 24,
            one_shot: true,
            particle_lifetime_min: 0.4,
            particle_lifetime_max: 1.1,
            initial_velocity_min: 3.0,
            initial_velocity_max: 9.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, -20.0, 0.0),
            drag: 0.6,
            size_start: 0.16,
            size_end: 0.06,
            emissive_strength: 0.6,
            color_gradient: fade(
                Vec4::new(0.54, 0.08, 0.10, 1.0),
                Vec4::new(0.30, 0.04, 0.05, 0.0),
            ),
            texture_index: data::SPARK_SLOT,
            ..Default::default()
        },
        1.4,
    );
}

pub fn explosion(commands: &mut Commands, at: Vec3, radius: f32) {
    burst(
        commands,
        at,
        ParticleEmitter {
            emitter_type: EmitterType::Fire,
            shape: EmitterShape::Sphere {
                radius: radius * 0.25,
            },
            position: at,
            direction: Vec3::y(),
            burst_count: 44,
            one_shot: true,
            particle_lifetime_min: 0.2,
            particle_lifetime_max: 0.6,
            initial_velocity_min: 4.0,
            initial_velocity_max: 16.0,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, 2.0, 0.0),
            drag: 2.2,
            size_start: radius * 0.4,
            size_end: radius * 0.12,
            emissive_strength: 9.0,
            texture_index: data::MUZZLE_SLOT,
            ..Default::default()
        },
        1.0,
    );
    burst(
        commands,
        at,
        ParticleEmitter {
            emitter_type: EmitterType::Smoke,
            shape: EmitterShape::Sphere {
                radius: radius * 0.3,
            },
            position: at,
            direction: Vec3::y(),
            burst_count: 20,
            one_shot: true,
            particle_lifetime_min: 0.6,
            particle_lifetime_max: 1.6,
            initial_velocity_min: 0.6,
            initial_velocity_max: 3.0,
            velocity_spread: 0.8,
            gravity: Vec3::new(0.0, 1.2, 0.0),
            drag: 2.6,
            size_start: radius * 0.3,
            size_end: radius * 0.9,
            emissive_strength: 0.4,
            texture_index: data::SMOKE_SLOT,
            ..Default::default()
        },
        2.0,
    );
    let flare = spawn_light_entity(commands, at, "boomstick_blast_light");
    commands.set(
        flare,
        Light {
            light_type: LightType::Point,
            color: Vec3::new(1.0, 0.58, 0.24),
            intensity: 260.0,
            range: radius * 4.0,
            cast_shadows: false,
            ..Default::default()
        },
    );
    commands.set(flare, Lifetime { remaining: 0.22 });
    commands.add_tag_type::<LevelPiece>(flare);
}

pub fn rocket_look(commands: &mut Commands, bolt: Entity) {
    commands.set(
        bolt,
        ParticleEmitter {
            emitter_type: EmitterType::Smoke,
            shape: EmitterShape::Point,
            position: Vec3::zeros(),
            direction: Vec3::y(),
            spawn_rate: 120.0,
            particle_lifetime_min: 0.25,
            particle_lifetime_max: 0.6,
            initial_velocity_min: 0.1,
            initial_velocity_max: 0.7,
            velocity_spread: 1.0,
            gravity: Vec3::zeros(),
            drag: 2.4,
            size_start: 0.22,
            size_end: 0.6,
            emissive_strength: 1.2,
            texture_index: data::SMOKE_SLOT,
            ..Default::default()
        },
    );
}

pub fn bolt_look(commands: &mut Commands, bolt: Entity, tint: Vec4) {
    commands.set(
        bolt,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Point,
            position: Vec3::zeros(),
            direction: Vec3::y(),
            spawn_rate: 200.0,
            particle_lifetime_min: 0.06,
            particle_lifetime_max: 0.2,
            initial_velocity_min: 0.2,
            initial_velocity_max: 1.0,
            velocity_spread: 1.0,
            gravity: Vec3::zeros(),
            drag: 3.0,
            size_start: 0.18,
            size_end: 0.02,
            emissive_strength: 7.0,
            color_gradient: fade(tint, Vec4::new(tint.x, tint.y, tint.z, 0.0)),
            texture_index: data::SPARK_SLOT,
            ..Default::default()
        },
    );
}

fn fade(from: Vec4, to: Vec4) -> ColorGradient {
    ColorGradient {
        colors: vec![(0.0, from), (1.0, to)],
    }
}

fn burst(commands: &mut Commands, at: Vec3, emitter: ParticleEmitter, life: f32) {
    let entity = commands.spawn((
        Name("Burst".to_string()),
        LocalTransform {
            translation: at,
            ..Default::default()
        },
        GlobalTransform::default(),
    ));
    commands.set(entity, emitter);
    commands.set(entity, Lifetime { remaining: life });
    commands.add_tag_type::<LevelPiece>(entity);
}
