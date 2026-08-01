use crate::components::{FacingLeft, Velocity};
use crate::data::Ask;
use crate::queries::solid_at;
use crate::resources::{CoinfallResources, Screen};
use crate::theme;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_platformer::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;
use nightshade_tiles::prelude::TileMap;

pub type Bodies<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Velocity,
        &'item mut Airborne,
        &'item mut Coyote,
        &'item mut JumpAsked,
        &'item mut FacingLeft,
        &'item mut LocalTransform,
        &'item mut Sprite,
        &'item mut SpriteAnimator,
    ),
>;

pub fn control(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    resources: Res<CoinfallResources>,
    settings: Res<PlatformerSettings>,
    mut bodies: Bodies<'_, '_>,
) {
    let running = resources.run.screen == Screen::Playing;
    let delta_time = time.delta_time.min(theme::MAX_STEP);
    let direction = if running {
        actions.axis(Ask::Steer)
    } else {
        0.0
    };
    let jump_held = running && actions.pressed(Ask::Jump);
    let jump_started = running && actions.just_pressed(Ask::Jump);
    let solids = &resources.solids;

    bodies.for_each_mut(
        |_entity, (velocity, airborne, coyote, asked, facing_left, transform, sprite, walk)| {
            if !running {
                velocity.0 = Vec2::zeros();
                hold_still(sprite, walk);
                return;
            }

            let target = direction * theme::RUN_SPEED;
            let acceleration = if direction == 0.0 {
                if airborne.0 {
                    theme::AIR_FRICTION
                } else {
                    theme::GROUND_FRICTION
                }
            } else if airborne.0 {
                theme::AIR_ACCELERATION
            } else if direction * velocity.0.x < 0.0 {
                theme::TURN_ACCELERATION
            } else {
                theme::GROUND_ACCELERATION
            };
            velocity.0.x = approach(velocity.0.x, target, acceleration * delta_time);

            if jump_started {
                ask_for_a_jump(asked, &settings);
            }
            if may_jump(coyote, asked) {
                velocity.0.y = theme::JUMP_SPEED;
                spend_the_jump(asked, coyote);
                airborne.0 = true;
            }

            if !jump_held && velocity.0.y > theme::SHORT_JUMP_SPEED {
                velocity.0.y = theme::SHORT_JUMP_SPEED;
            }

            let gravity = if velocity.0.y > 0.0 {
                theme::RISE_GRAVITY
            } else {
                theme::FALL_GRAVITY
            };
            velocity.0.y = (velocity.0.y - gravity * delta_time).max(-theme::TERMINAL_SPEED);

            let mut position = Vec2::new(transform.translation.x, transform.translation.y);
            position.x += velocity.0.x * delta_time;
            if push_out_sideways(solids, &mut position, velocity.0.x) {
                velocity.0.x = 0.0;
            }
            position.y += velocity.0.y * delta_time;
            airborne.0 = !push_out_upright(solids, &mut position, &mut velocity.0.y);
            transform.translation.x = position.x;
            transform.translation.y = position.y;

            if velocity.0.x.abs() > 0.2 {
                facing_left.0 = velocity.0.x < 0.0;
            }
            sprite.flip_x = facing_left.0;

            if !airborne.0 && velocity.0.x.abs() > 0.5 {
                if !walk.playing {
                    walk.play();
                }
            } else {
                hold_still(sprite, walk);
            }
        },
    );
}

fn hold_still(sprite: &mut Sprite, walk: &mut SpriteAnimator) {
    if walk.playing {
        walk.pause();
    }
    walk.reset();
    if let Some(region) = walk.current_region() {
        sprite.region = region;
    }
}

fn approach(value: f32, target: f32, step: f32) -> f32 {
    if value < target {
        (value + step).min(target)
    } else {
        (value - step).max(target)
    }
}

fn push_out_sideways(solids: &TileMap, position: &mut Vec2, velocity_x: f32) -> bool {
    let half = theme::PLAYER_HALF_WIDTH;
    let bottom = position.y + theme::SKIN;
    let top = position.y + theme::PLAYER_HEIGHT - theme::SKIN;
    let first_row = bottom.floor() as i32;
    let last_row = top.floor() as i32;

    if velocity_x > 0.0 {
        let column = (position.x + half).floor() as i32;
        for row in first_row..=last_row {
            if solid_at(solids, column, row) {
                position.x = column as f32 - half - theme::SKIN;
                return true;
            }
        }
    } else if velocity_x < 0.0 {
        let column = (position.x - half).floor() as i32;
        for row in first_row..=last_row {
            if solid_at(solids, column, row) {
                position.x = (column + 1) as f32 + half + theme::SKIN;
                return true;
            }
        }
    }
    false
}

fn push_out_upright(solids: &TileMap, position: &mut Vec2, velocity_y: &mut f32) -> bool {
    let half = theme::PLAYER_HALF_WIDTH;
    let left = position.x - half + theme::SKIN;
    let right = position.x + half - theme::SKIN;
    let first_column = left.floor() as i32;
    let last_column = right.floor() as i32;

    if *velocity_y <= 0.0 {
        let row = (position.y - theme::SKIN).floor() as i32;
        for column in first_column..=last_column {
            if solid_at(solids, column, row) {
                position.y = (row + 1) as f32;
                *velocity_y = 0.0;
                return true;
            }
        }
        return false;
    }

    let row = (position.y + theme::PLAYER_HEIGHT).floor() as i32;
    for column in first_column..=last_column {
        if solid_at(solids, column, row) {
            position.y = row as f32 - theme::PLAYER_HEIGHT - theme::SKIN;
            *velocity_y = 0.0;
            return false;
        }
    }
    false
}
