use crate::components::{Alive, Direction, FacingLeft, Speed, Velocity};
use crate::contract::CoinfallEvent;
use crate::queries::{boxes_overlap, solid_at};
use crate::resources::{CoinfallResources, Screen};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_platformer::{Airborne, JumpAsked};
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub type Bodies<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Speed,
        &'item mut Direction,
        &'item mut Alive,
        &'item mut LocalTransform,
        &'item mut Sprite,
    ),
>;

pub type Marks<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Velocity,
        &'item JumpAsked,
        &'item mut Airborne,
        &'item FacingLeft,
        &'item LocalTransform,
    ),
>;

pub fn patrol(time: Res<Time>, resources: Res<CoinfallResources>, mut bodies: Bodies<'_, '_>) {
    if resources.run.screen == Screen::Paused {
        return;
    }
    let delta_time = time.delta_time.min(theme::MAX_STEP);
    let solids = &resources.solids;

    bodies.for_each_mut(|_entity, (speed, direction, alive, transform, sprite)| {
        if !alive.0 {
            return;
        }
        let step = direction.0 * speed.0 * delta_time;
        let next_x = transform.translation.x + step;
        let edge_x = next_x + direction.0 * theme::WALKER_HALF_WIDTH;
        let ahead = edge_x.floor() as i32;
        let body_row = (transform.translation.y + theme::WALKER_HEIGHT * 0.5).floor() as i32;
        let floor_row = (transform.translation.y - theme::SKIN).floor() as i32;

        if solid_at(solids, ahead, body_row) || !solid_at(solids, ahead, floor_row) {
            direction.0 = -direction.0;
        } else {
            transform.translation.x = next_x;
        }
        sprite.flip_x = direction.0 > 0.0;
    });
}

pub fn stomp(
    resources: Res<CoinfallResources>,
    mut events: EventWriter<CoinfallEvent>,
    mut commands: Commands,
    mut marks: Marks<'_, '_>,
    mut bodies: Bodies<'_, '_>,
) {
    if resources.run.screen != Screen::Playing {
        return;
    }
    let mut mark = None;
    marks.for_each_mut(|_entity, (velocity, _, _, _, transform)| {
        mark = Some((transform.translation, velocity.0));
    });
    let Some((position, velocity)) = mark else {
        return;
    };

    let player_center = Vec2::new(position.x, position.y + theme::PLAYER_HEIGHT * 0.5);
    let player_half = Vec2::new(theme::PLAYER_HALF_WIDTH, theme::PLAYER_HEIGHT * 0.5);
    let mut stomped = false;
    let mut struck = false;

    bodies.for_each_mut(|entity, (_speed, _direction, alive, transform, _)| {
        if !alive.0 {
            return;
        }
        let walker_center = Vec2::new(
            transform.translation.x,
            transform.translation.y + theme::WALKER_HEIGHT * 0.5,
        );
        let walker_half = Vec2::new(theme::WALKER_HALF_WIDTH, theme::WALKER_HEIGHT * 0.5);
        if !boxes_overlap(player_center, player_half, walker_center, walker_half) {
            return;
        }
        let from_above = velocity.y < 0.0
            && position.y > transform.translation.y + theme::WALKER_HEIGHT - theme::STOMP_MARGIN;
        if from_above {
            alive.0 = false;
            commands.set(entity, Visibility { visible: false });
            stomped = true;
        } else {
            struck = true;
        }
    });

    if !stomped && !struck {
        return;
    }

    marks.for_each_mut(|_entity, (velocity, asked, airborne, facing_left, _)| {
        if stomped {
            velocity.0.y = if asked.0 > 0.0 {
                theme::STOMP_HELD_SPEED
            } else {
                theme::STOMP_SPEED
            };
            airborne.0 = true;
        }
        if struck {
            velocity.0.x = if facing_left.0 {
                theme::HIT_KNOCKBACK
            } else {
                -theme::HIT_KNOCKBACK
            };
            velocity.0.y = velocity.0.y.max(theme::HIT_LIFT);
            airborne.0 = true;
        }
    });

    if stomped {
        events.send(CoinfallEvent::WalkerStomped);
    }
    if struck {
        events.send(CoinfallEvent::WalkerHit);
    }
}
