use crate::commands::life::spawn_standing;
use crate::data::{self, Placement};
use crate::queries::sheet::standing;
use crate::resources::Sheet;
use crate::tags::Bobber;
use nalgebra_glm::{Vec2, Vec3, Vec4};
use nightshade::prelude::*;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};

pub fn spawn_bobber(
    commands: &mut Commands,
    sheet: &Sheet,
    turntable: Entity,
    spot: Vec2,
    lift: f32,
) -> Entity {
    let placement = Placement::new(spot, lift).with_scale(0.4);
    let bobber = spawn_standing(
        commands,
        sheet,
        turntable,
        standing(sheet, data::BOBBER_TILE),
        placement,
    );
    commands.add_tag_type::<Bobber>(bobber);
    bobber
}

pub fn spawn_rain(commands: &mut Commands, position: Vec3) -> Entity {
    let entity = commands.spawn(());
    commands.set(
        entity,
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Box {
                half_extents: Vec3::new(11.0, 0.5, 9.0),
            },
            position,
            direction: Vec3::new(0.0, -1.0, 0.0),
            spawn_rate: data::RAIN_RATE,
            burst_count: 0,
            particle_lifetime_min: 0.85,
            particle_lifetime_max: 1.05,
            initial_velocity_min: 11.0,
            initial_velocity_max: 13.0,
            velocity_spread: 0.03,
            gravity: Vec3::new(0.0, 0.0, 0.0),
            drag: 0.0,
            size_start: 0.08,
            size_end: 0.08,
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
        },
    );
    entity
}
