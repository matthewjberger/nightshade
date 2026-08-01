use crate::components::{Armor, Grow, HalfExtent, Points, Pop, Radius, Speed, Squash, Velocity};
use crate::contract::BreakoutAdvancedEvent;
use crate::data;
use crate::data::Ask;
use crate::queries;
use crate::resources::BreakoutAdvancedResources;
use crate::systems::setup::brick_material;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

const LONGEST_FRAME: f32 = 1.0 / 30.0;
const SERVE_SPREAD: f32 = 0.6;

type Standing<'world, 'item> = Query<
    'world,
    (
        &'item HalfExtent,
        &'item Points,
        &'item Armor,
        &'item LocalTransform,
    ),
    (Without<Pop>, Without<Grow>),
>;

pub fn advance(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    mut resources: ResMut<BreakoutAdvancedResources>,
    mut events: EventWriter<BreakoutAdvancedEvent>,
    mut commands: Commands,
    mut paddles: Query<(&HalfExtent, &LocalTransform), With<Speed>>,
    mut brick_boxes: Standing<'_, '_>,
    balls: Query<(&Radius, &mut Velocity, &mut LocalTransform)>,
) {
    let mut paddle = None;
    paddles.for_each_mut(|_entity, (half_extent, transform)| {
        paddle = Some((
            Vec2::new(transform.translation.x, transform.translation.y),
            half_extent.0,
        ));
    });
    let Some((paddle_center, paddle_half)) = paddle else {
        return;
    };

    let mut bricks = Vec::new();
    brick_boxes.for_each_mut(|entity, (half_extent, points, armor, transform)| {
        bricks.push((
            entity,
            Vec2::new(transform.translation.x, transform.translation.y),
            half_extent.0,
            points.0,
            armor.0,
        ));
    });

    if resources.round.serving {
        serve(&mut resources, &actions, paddle_center, balls);
        return;
    }

    let delta_time = time.delta_time.min(LONGEST_FRAME) * resources.juice.time_scale;
    let mut struck: Vec<(Entity, Vec2, u32, u32)> = Vec::new();
    let mut wall_hits: Vec<Vec2> = Vec::new();
    let mut paddle_hits: Vec<(Vec2, f32)> = Vec::new();
    let mut lost = false;

    balls.for_each(|_entity, (radius, velocity, transform)| {
        let mut position = Vec2::new(transform.translation.x, transform.translation.y);
        let travel = velocity.0.magnitude() * delta_time;
        let steps = (travel / data::BALL_STEP).ceil().max(1.0) as u32;
        let step_time = delta_time / steps as f32;

        for _ in 0..steps {
            position += velocity.0 * step_time;

            let side = data::INNER_HALF_WIDTH - radius.0;
            if position.x < -side {
                position.x = -side;
                velocity.0.x = velocity.0.x.abs();
                wall_hits.push(position);
            } else if position.x > side {
                position.x = side;
                velocity.0.x = -velocity.0.x.abs();
                wall_hits.push(position);
            }

            let ceiling = data::INNER_TOP - radius.0;
            if position.y > ceiling {
                position.y = ceiling;
                velocity.0.y = -velocity.0.y.abs();
                wall_hits.push(position);
            }

            if velocity.0.y < 0.0
                && queries::separation(position, radius.0, paddle_center, paddle_half).is_some()
            {
                let offset = (position.x - paddle_center.x) / paddle_half.x;
                velocity.0 = queries::bounce_direction(offset) * data::BALL_SPEED;
                position.y = paddle_center.y + paddle_half.y + radius.0;
                paddle_hits.push((position, offset));
            }

            let mut found = None;
            for (entity, center, half_extent, points, armor) in bricks.iter() {
                if struck.iter().any(|(already, _, _, _)| already == entity) {
                    continue;
                }
                if let Some(normal) = queries::separation(position, radius.0, *center, *half_extent)
                {
                    found = Some((
                        *entity,
                        *center,
                        *points,
                        *armor,
                        queries::axis_normal(normal),
                    ));
                    break;
                }
            }
            if let Some((entity, center, points, armor, axis)) = found {
                if axis.x != 0.0 {
                    velocity.0.x = axis.x * velocity.0.x.abs();
                }
                if axis.y != 0.0 {
                    velocity.0.y = axis.y * velocity.0.y.abs();
                }
                struck.push((entity, center, points, armor));
            }

            if position.y < data::FLOOR_Y {
                lost = true;
                break;
            }
        }

        transform.translation = Vec3::new(position.x, position.y, 0.0);
    });

    for at in wall_hits {
        events.send(BreakoutAdvancedEvent::WallHit { at: [at.x, at.y] });
    }
    for (at, offset) in paddle_hits {
        events.send(BreakoutAdvancedEvent::PaddleHit {
            at: [at.x, at.y],
            offset,
        });
    }
    for (entity, center, points, armor) in struck {
        if armor > 0 {
            let left = armor - 1;
            commands.set(entity, Armor(left));
            set_mesh_material(
                &mut commands,
                entity,
                &brick_material(queries::row_of(center.y), left),
            );
            crate::commands::chip_brick(&mut resources.round);
            events.send(BreakoutAdvancedEvent::BrickChipped {
                at: [center.x, center.y],
                armor: left,
            });
            continue;
        }
        commands.set(entity, Pop(data::POP_SECONDS));
        crate::commands::score_points(&mut resources.round, points);
        events.send(BreakoutAdvancedEvent::BrickBroken {
            at: [center.x, center.y],
            points,
        });
    }

    if lost {
        crate::commands::lose_life(&mut resources.round);
        events.send(BreakoutAdvancedEvent::BallLost);
    }

    if resources.round.bricks_standing == 0 && !resources.round.rebuild_wanted {
        let level = resources.round.level;
        let bonus = crate::commands::clear_level(&mut resources.round);
        events.send(BreakoutAdvancedEvent::LevelCleared { level, bonus });
    }
}

pub fn shape(
    time: Res<Time>,
    balls: Query<(&Radius, &Velocity, &mut Squash, &mut LocalTransform)>,
) {
    let delta_time = time.delta_time;
    balls.for_each(|_entity, (radius, velocity, squash, transform)| {
        squash.0 = (squash.0 - squash.0 * data::SQUASH_DECAY * delta_time).max(0.0);
        let shape = queries::ball_shape(velocity.0, squash.0);
        let angle = if velocity.0.magnitude() > f32::EPSILON {
            velocity.0.y.atan2(velocity.0.x)
        } else {
            0.0
        };
        transform.rotation = nalgebra_glm::quat_angle_axis(angle, &Vec3::z());
        transform.scale = Vec3::new(radius.0 * shape.x, radius.0 * shape.y, radius.0);
    });
}

fn serve(
    resources: &mut BreakoutAdvancedResources,
    actions: &Actions<Ask>,
    paddle_center: Vec2,
    balls: Query<'_, (&Radius, &mut Velocity, &mut LocalTransform)>,
) {
    let launching = actions.just_pressed(Ask::Serve);
    let launch = launching.then(|| {
        let drift = crate::commands::next_random(&mut resources.round) * SERVE_SPREAD;
        queries::bounce_direction(drift) * data::BALL_SPEED
    });

    let at = queries::serve_position(paddle_center.x);
    balls.for_each(|_entity, (_radius, velocity, transform)| match launch {
        Some(direction) => velocity.0 = direction,
        None => {
            transform.translation = Vec3::new(at.x, at.y, 0.0);
            velocity.0 = Vec2::zeros();
        }
    });

    if launching {
        resources.round.serving = false;
    }
}
