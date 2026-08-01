use crate::commands::{hold_slow, shake, tint_toward};
use crate::components::Squash;
use crate::contract::BreakoutAdvancedEvent;
use crate::data;
use crate::queries;
use crate::resources::BreakoutAdvancedResources;
use nalgebra_glm::{Vec3, Vec4};
use nightshade::prelude::*;
use nightshade_graphics::prelude::RenderSettings;
use nightshade_particles::contract::{ColorGradient, EmitterShape, EmitterType, ParticleEmitter};
use nightshade_platform::prelude::*;
use nightshade_scene::contract::Lifetime;
use nightshade_scene::prelude::*;

pub fn react(
    events: EventReader<BreakoutAdvancedEvent>,
    mut resources: ResMut<BreakoutAdvancedResources>,
    mut commands: Commands,
    balls: Query<(&mut Squash,)>,
) {
    let resources = &mut *resources;
    let mut impact = 0.0_f32;

    for event in &events {
        match event {
            BreakoutAdvancedEvent::WallHit { .. } => {
                shake(&mut resources.juice, data::SHAKE_WALL);
                impact = impact.max(0.45);
            }
            BreakoutAdvancedEvent::PaddleHit { .. } => {
                shake(&mut resources.juice, data::SHAKE_PADDLE);
                impact = impact.max(0.8);
            }
            BreakoutAdvancedEvent::BrickChipped { at, armor } => {
                let color = queries::tier_color(queries::row_of(at[1]), *armor);
                shake(&mut resources.juice, data::SHAKE_CHIP);
                tint_toward(&mut resources.juice, color);
                impact = impact.max(0.9);
                spawn_debris(&mut commands, Vec3::new(at[0], at[1], 0.0), color);
            }
            BreakoutAdvancedEvent::BrickBroken { at, .. } => {
                let color = queries::row_color(queries::row_of(at[1]));
                shake(&mut resources.juice, data::SHAKE_BRICK);
                tint_toward(&mut resources.juice, color);
                impact = impact.max(1.0);
                spawn_debris(&mut commands, Vec3::new(at[0], at[1], 0.0), color);
                if resources.round.bricks_standing == 0 {
                    hold_slow(&mut resources.juice);
                }
            }
            BreakoutAdvancedEvent::BallLost => {
                shake(&mut resources.juice, data::SHAKE_LOST);
            }
            BreakoutAdvancedEvent::LevelCleared { bonus, .. } => {
                hold_slow(&mut resources.juice);
                if *bonus > 0 {
                    shake(&mut resources.juice, data::SHAKE_LOST);
                    tint_toward(&mut resources.juice, data::BALL_COLOR);
                }
            }
        }
    }

    if impact <= 0.0 {
        return;
    }
    balls.for_each(|_entity, (squash,)| squash.0 = squash.0.max(impact));
}

pub fn settle(
    time: Res<Time>,
    mut resources: ResMut<BreakoutAdvancedResources>,
    mut settings: ResMut<RenderSettings>,
) {
    let delta_time = time.delta_time;
    let juice = &mut resources.juice;

    juice.shake = (juice.shake - juice.shake * data::SHAKE_DECAY * delta_time).max(0.0);

    let drift = (data::TINT_DRIFT * delta_time).clamp(0.0, 1.0);
    juice.tint += (juice.tint_target - juice.tint) * drift;
    juice.tint_target -= juice.tint_target * drift * 0.5;

    let wanted_zoom = queries::zoom_target(resources.round.bricks_standing);
    let ease = (data::ZOOM_EASE * delta_time).clamp(0.0, 1.0);
    let juice = &mut resources.juice;
    juice.zoom += (wanted_zoom - juice.zoom) * ease;

    juice.slow_hold = (juice.slow_hold - delta_time).max(0.0);
    let wanted_scale = if juice.slow_hold > 0.0 {
        data::SLOW_SCALE
    } else {
        1.0
    };
    let pace = (data::SLOW_EASE * delta_time).clamp(0.0, 1.0);
    juice.time_scale += (wanted_scale - juice.time_scale) * pace;

    settings.clear_color = queries::background(juice.tint);
}

fn spawn_debris(commands: &mut Commands, at: Vec3, color: [f32; 4]) {
    let bright = Vec4::new(color[0], color[1], color[2], 1.0);
    let faded = Vec4::new(color[0], color[1], color[2], 0.0);
    let entity = commands.spawn((
        Name("Brick Debris".to_string()),
        ParticleEmitter {
            emitter_type: EmitterType::Sparks,
            shape: EmitterShape::Sphere {
                radius: data::BRICK_HALF_HEIGHT,
            },
            position: at,
            direction: Vec3::new(0.0, 1.0, 0.0),
            spawn_rate: 0.0,
            burst_count: data::DEBRIS_COUNT,
            particle_lifetime_min: data::DEBRIS_LIFE.0,
            particle_lifetime_max: data::DEBRIS_LIFE.1,
            initial_velocity_min: data::DEBRIS_SPEED.0,
            initial_velocity_max: data::DEBRIS_SPEED.1,
            velocity_spread: 1.0,
            gravity: Vec3::new(0.0, data::DEBRIS_GRAVITY, 0.0),
            drag: 0.6,
            size_start: data::DEBRIS_SIZE.0,
            size_end: data::DEBRIS_SIZE.1,
            color_gradient: ColorGradient {
                colors: vec![
                    (0.0, Vec4::new(1.0, 1.0, 1.0, 1.0)),
                    (0.25, bright),
                    (1.0, faded),
                ],
            },
            emissive_strength: 2.0,
            one_shot: true,
            ..Default::default()
        },
    ));
    commands.set(
        entity,
        Lifetime {
            remaining: data::DEBRIS_LIFE.1 + 0.3,
        },
    );
}
