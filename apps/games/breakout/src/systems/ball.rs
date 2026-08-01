use crate::components::{HalfExtent, Points, Radius, Speed, Velocity};
use crate::contract::BreakoutEvent;
use crate::data;
use crate::data::Ask;
use crate::queries;
use crate::resources::RoundState;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn advance(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    mut round: ResMut<RoundState>,
    mut events: EventWriter<BreakoutEvent>,
    mut commands: Commands,
    mut paddles: Query<(&HalfExtent, &LocalTransform), With<Speed>>,
    mut brick_boxes: Query<(&HalfExtent, &Points, &LocalTransform)>,
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

    if round.serving {
        serve(&mut round, &actions, paddle_center, balls);
        return;
    }

    let mut bricks = Vec::new();
    brick_boxes.for_each_mut(|entity, (half_extent, points, transform)| {
        bricks.push((
            entity,
            Vec2::new(transform.translation.x, transform.translation.y),
            half_extent.0,
            points.0,
        ));
    });

    let delta_time = time.delta_time.min(data::LONGEST_FRAME);
    let mut broken: Vec<(Entity, u32)> = Vec::new();
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
            } else if position.x > side {
                position.x = side;
                velocity.0.x = -velocity.0.x.abs();
            }

            let ceiling = data::INNER_TOP - radius.0;
            if position.y > ceiling {
                position.y = ceiling;
                velocity.0.y = -velocity.0.y.abs();
            }

            if velocity.0.y < 0.0
                && queries::separation(position, radius.0, paddle_center, paddle_half).is_some()
            {
                let offset = (position.x - paddle_center.x) / paddle_half.x;
                velocity.0 = queries::bounce_direction(offset) * data::BALL_SPEED;
                position.y = paddle_center.y + paddle_half.y + radius.0;
            }

            let mut struck = None;
            for (index, (_, center, half_extent, _)) in bricks.iter().enumerate() {
                if let Some(normal) = queries::separation(position, radius.0, *center, *half_extent)
                {
                    struck = Some((index, queries::axis_normal(normal)));
                    break;
                }
            }
            if let Some((index, axis)) = struck {
                if axis.x != 0.0 {
                    velocity.0.x = axis.x * velocity.0.x.abs();
                } else {
                    velocity.0.y = axis.y * velocity.0.y.abs();
                }
                let (entity, _, _, points) = bricks.swap_remove(index);
                broken.push((entity, points));
            }

            if position.y < data::FLOOR_Y {
                lost = true;
                break;
            }
        }

        transform.translation = Vec3::new(position.x, position.y, 0.0);
    });

    for (entity, points) in broken {
        commands.despawn(entity);
        events.send(BreakoutEvent::BrickBroken { points });
    }

    if lost {
        round.serving = true;
        events.send(BreakoutEvent::BallLost);
    }
}

fn serve(
    round: &mut RoundState,
    actions: &Actions<Ask>,
    paddle_center: Vec2,
    balls: Query<'_, (&Radius, &mut Velocity, &mut LocalTransform)>,
) {
    let launching = actions.just_pressed(Ask::Serve);
    let launch = launching.then(|| {
        let drift = crate::commands::next_random(round) * data::SERVE_SPREAD;
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
        round.serving = false;
    }
}
