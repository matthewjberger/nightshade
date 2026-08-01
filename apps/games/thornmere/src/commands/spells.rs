use crate::commands::scenery::prop;
use crate::components::{Age, Aura, Form, Home, Lift, Peak, Span, Spin};
use crate::data::{CHILL_SECONDS, Shape};
use crate::resources::Palette;
use nightshade::prelude::*;
use nightshade_graphics::prelude::{Light, LightType};
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};
use nightshade_random::prelude::*;
use nightshade_render::data::asset_id::MaterialId;
use nightshade_scene::prelude::{GlobalTransform, Lifetime, LocalTransform, Name};

const TAU: f32 = std::f32::consts::TAU;

pub fn shape(
    commands: &mut Commands,
    mesh: &str,
    material: MaterialId,
    position: Vec3,
    peak: f32,
    form: Shape,
    span: f32,
) -> Entity {
    let entity = prop(
        commands,
        mesh,
        position,
        Vec3::new(0.01, 0.01, 0.01),
        material,
    );
    commands.set(entity, Age(0.0));
    commands.set(entity, Span(span));
    commands.set(entity, Form(form));
    commands.set(entity, Peak(peak));
    commands.set(entity, Spin(0.0));
    commands.set(entity, Home(position));
    commands.set(entity, Lift(0.0));
    entity
}

pub fn lamp(
    commands: &mut Commands,
    position: Vec3,
    color: Vec3,
    intensity: f32,
    range: f32,
    span: f32,
) -> Entity {
    let transform = LocalTransform {
        translation: position,
        ..Default::default()
    };
    let entity = commands.spawn((
        Name("Spell Light".to_string()),
        transform,
        GlobalTransform(transform.as_matrix()),
        Light {
            light_type: LightType::Point,
            color,
            intensity: 0.0,
            range,
            ..Default::default()
        },
    ));
    commands.set(entity, Age(0.0));
    commands.set(entity, Span(span));
    commands.set(entity, Form(Shape::Lamp));
    commands.set(entity, Peak(intensity));
    entity
}

pub fn bolt_light(commands: &mut Commands, bolt: Entity, color: Vec3, intensity: f32) {
    commands.set(
        bolt,
        Light {
            light_type: LightType::Point,
            color,
            intensity,
            range: 7.0,
            ..Default::default()
        },
    );
}

pub fn fire_blast(commands: &mut Commands, palette: &Palette, at: Vec3) {
    shape(commands, "Sphere", palette.fire, at, 2.4, Shape::Blast, 0.7);
    shape(
        commands,
        "Cylinder",
        palette.soot,
        Vec3::new(at.x, at.y - 1.1, at.z),
        2.8,
        Shape::Patch,
        7.0,
    );
    lamp(commands, at, Vec3::new(1.0, 0.55, 0.2), 45.0, 10.0, 0.8);
}

pub fn frost_shatter(commands: &mut Commands, palette: &Palette, random: &mut Random, at: Vec3) {
    let foot = Vec3::new(at.x, at.y - 1.1, at.z);
    for index in 0..7 {
        let angle = index as f32 / 7.0 * TAU + random.f32() * 0.6;
        let radius = 0.5 + random.f32() * 0.8;
        let spot = foot + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);
        let shard = shape(
            commands,
            "Cone",
            palette.ice,
            spot,
            1.2 + random.f32() * 1.2,
            Shape::Shard,
            3.2 + random.f32(),
        );
        commands.set(shard, Spin(random.f32() * 0.5 - 0.25));
        commands.set(shard, Lift(angle));
    }
    shape(
        commands,
        "Cylinder",
        palette.ice,
        foot,
        2.6,
        Shape::Patch,
        5.0,
    );
    shape(
        commands,
        "Sphere",
        palette.frost,
        at,
        1.6,
        Shape::Blast,
        0.5,
    );
    lamp(commands, at, Vec3::new(0.5, 0.75, 1.0), 32.0, 9.0, 1.2);
}

pub fn slash(commands: &mut Commands, palette: &Palette, at: Vec3, yaw: f32) {
    let arc = shape(commands, "Torus", palette.holy, at, 1.7, Shape::Slash, 0.4);
    commands.set(arc, Spin(yaw));
    lamp(commands, at, Vec3::new(1.0, 0.9, 0.6), 24.0, 6.0, 0.35);
}

pub fn whirl(commands: &mut Commands, palette: &Palette, at: Vec3) {
    let outer = shape(
        commands,
        "Torus",
        palette.holy,
        at + Vec3::new(0.0, 1.0, 0.0),
        3.6,
        Shape::Whirl,
        0.85,
    );
    commands.set(outer, Spin(14.0));
    let inner = shape(
        commands,
        "Torus",
        palette.fire,
        at + Vec3::new(0.0, 1.3, 0.0),
        2.4,
        Shape::Whirl,
        0.7,
    );
    commands.set(inner, Spin(-11.0));
    lamp(
        commands,
        at + Vec3::new(0.0, 1.2, 0.0),
        Vec3::new(1.0, 0.85, 0.5),
        34.0,
        9.0,
        0.9,
    );
}

pub fn halo(commands: &mut Commands, palette: &Palette, at: Vec3) {
    let ring = shape(commands, "Torus", palette.life, at, 1.5, Shape::Halo, 1.1);
    commands.set(ring, Lift(2.2));
    commands.set(ring, Spin(3.0));
    shape(
        commands,
        "Sphere",
        palette.life,
        at + Vec3::new(0.0, 1.2, 0.0),
        1.4,
        Shape::Blast,
        0.6,
    );
    lamp(
        commands,
        at + Vec3::new(0.0, 1.2, 0.0),
        Vec3::new(0.55, 1.0, 0.6),
        22.0,
        7.0,
        1.1,
    );
}

pub fn ascension(commands: &mut Commands, palette: &Palette, at: Vec3) {
    let ring = shape(
        commands,
        "Torus",
        palette.holy,
        at + Vec3::new(0.0, 0.1, 0.0),
        5.5,
        Shape::Slash,
        1.3,
    );
    commands.set(ring, Spin(0.0));
    let halo_ring = shape(commands, "Torus", palette.holy, at, 1.8, Shape::Halo, 1.6);
    commands.set(halo_ring, Lift(3.5));
    commands.set(halo_ring, Spin(2.0));
    lamp(
        commands,
        at + Vec3::new(0.0, 1.5, 0.0),
        Vec3::new(1.0, 0.85, 0.4),
        70.0,
        14.0,
        1.6,
    );
}

pub fn requiem(commands: &mut Commands, palette: &Palette, at: Vec3, elite: bool) {
    if elite {
        let ring = shape(
            commands,
            "Torus",
            palette.violet,
            at + Vec3::new(0.0, 0.1, 0.0),
            6.0,
            Shape::Slash,
            1.5,
        );
        commands.set(ring, Spin(0.0));
        lamp(
            commands,
            at + Vec3::new(0.0, 1.5, 0.0),
            Vec3::new(0.7, 0.4, 1.0),
            60.0,
            14.0,
            1.8,
        );
    } else {
        lamp(
            commands,
            at + Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 0.3, 0.2),
            14.0,
            5.0,
            0.4,
        );
    }
}

fn aura(
    commands: &mut Commands,
    target: Entity,
    height: f32,
    emitter: ParticleEmitter,
    seconds: f32,
) {
    let entity = commands.spawn(());
    commands.set(entity, emitter);
    commands.set(entity, Aura(Some(target)));
    commands.set(entity, Lift(height));
    commands.set(entity, Lifetime { remaining: seconds });
}

pub fn chill_aura(commands: &mut Commands, target: Entity) {
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Sphere { radius: 0.5 },
        direction: Vec3::new(0.0, 1.0, 0.0),
        spawn_rate: 26.0,
        particle_lifetime_min: 0.5,
        particle_lifetime_max: 1.0,
        initial_velocity_min: 0.1,
        initial_velocity_max: 0.5,
        velocity_spread: std::f32::consts::PI,
        gravity: Vec3::new(0.0, -0.8, 0.0),
        drag: 1.0,
        size_start: 0.09,
        size_end: 0.0,
        color_gradient: ColorGradient {
            colors: vec![
                (0.0, Vec4::new(0.95, 1.0, 1.0, 1.0)),
                (0.5, Vec4::new(0.6, 0.85, 1.0, 0.9)),
                (1.0, Vec4::new(0.4, 0.6, 1.0, 0.0)),
            ],
        },
        emissive_strength: 5.0,
        ..Default::default()
    };
    aura(commands, target, 1.0, emitter, CHILL_SECONDS);
}

pub fn sprint_trail(commands: &mut Commands, target: Entity, seconds: f32) {
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Sphere { radius: 0.35 },
        direction: Vec3::new(0.0, 1.0, 0.0),
        spawn_rate: 40.0,
        particle_lifetime_min: 0.3,
        particle_lifetime_max: 0.7,
        initial_velocity_min: 0.2,
        initial_velocity_max: 0.9,
        velocity_spread: std::f32::consts::PI,
        gravity: Vec3::new(0.0, 0.5, 0.0),
        drag: 1.4,
        size_start: 0.2,
        size_end: 0.0,
        color_gradient: ColorGradient {
            colors: vec![
                (0.0, Vec4::new(0.75, 0.68, 0.55, 0.5)),
                (0.5, Vec4::new(0.65, 0.6, 0.5, 0.35)),
                (1.0, Vec4::new(0.5, 0.45, 0.4, 0.0)),
            ],
        },
        emissive_strength: 0.0,
        ..Default::default()
    };
    aura(commands, target, 0.2, emitter, seconds);
}

pub fn heal_aura(commands: &mut Commands, target: Entity) {
    let emitter = ParticleEmitter {
        emitter_type: EmitterType::Sparks,
        shape: EmitterShape::Sphere { radius: 0.6 },
        direction: Vec3::new(0.0, 1.0, 0.0),
        spawn_rate: 40.0,
        particle_lifetime_min: 0.6,
        particle_lifetime_max: 1.2,
        initial_velocity_min: 0.6,
        initial_velocity_max: 1.4,
        velocity_spread: 0.6,
        gravity: Vec3::new(0.0, 1.0, 0.0),
        drag: 0.8,
        size_start: 0.1,
        size_end: 0.0,
        color_gradient: ColorGradient {
            colors: vec![
                (0.0, Vec4::new(0.9, 1.0, 0.85, 1.0)),
                (0.5, Vec4::new(0.5, 1.0, 0.55, 0.9)),
                (1.0, Vec4::new(0.3, 0.8, 0.4, 0.0)),
            ],
        },
        emissive_strength: 5.0,
        ..Default::default()
    };
    aura(commands, target, 0.3, emitter, 1.2);
}
