use crate::components::{Rest, Speed, Stride};
use crate::data::Ask;
use crate::queries::camera::ground_step;
use crate::queries::interior::can_step as can_step_indoors;
use crate::queries::terrain::{can_step, height_at};
use crate::resources::{Diorama, Dungeon, Fishing, FishingState, Hero, Home, Rig, Terrain};
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn walk_the_farmer(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    terrain: Res<Terrain>,
    fishing: Res<Fishing>,
    dungeon: Res<Dungeon>,
    home: Res<Home>,
    hero: Res<Hero>,
    rig: Res<Rig>,
    mut diorama: ResMut<Diorama>,
    farmers: Query<(
        &Speed,
        &mut Stride,
        &mut Rest,
        &mut LocalTransform,
        &mut Sprite,
    )>,
) {
    let mut direction = Vec2::zeros();
    if fishing.state == FishingState::Idle {
        let walk = actions.axis2(Ask::Walk);
        direction.x += walk.x;
        direction.y -= walk.y + actions.axis(Ask::WalkAlong);
    }
    let moving = direction.magnitude_squared() > 0.0;
    let turning = direction.x != 0.0;
    let facing_left = direction.x < 0.0;
    if moving {
        direction = ground_step(rig.yaw, direction.normalize());
    }
    let delta_time = time.delta_time;
    let mut feet = None;
    let below = dungeon.active;
    let inside = home.active;
    let haste = if hero.speed > 0.0 {
        1.0 + theme::SPEED_BONUS
    } else {
        1.0
    };

    farmers.for_each(|_entity, (speed, stride, rest, transform, sprite)| {
        rest.0 = (rest.0 - delta_time).max(0.0);
        let mut ground = Vec2::new(transform.translation.x, transform.translation.z);
        if moving {
            let step = direction * speed.0 * haste * delta_time;
            let along_x = Vec2::new(ground.x + step.x, ground.y);
            let allowed_x = if inside {
                can_step_indoors(&home, along_x)
            } else if below {
                crate::queries::dungeon::can_step(&dungeon, along_x)
            } else {
                can_step(&terrain, ground, along_x)
            };
            if allowed_x {
                ground = along_x;
            }
            let along_z = Vec2::new(ground.x, ground.y + step.y);
            let allowed_z = if inside {
                can_step_indoors(&home, along_z)
            } else if below {
                crate::queries::dungeon::can_step(&dungeon, along_z)
            } else {
                can_step(&terrain, ground, along_z)
            };
            if allowed_z {
                ground = along_z;
            }
            transform.translation.x = ground.x;
            transform.translation.z = ground.y;
            if turning {
                sprite.flip_x = facing_left;
            }
            stride.0 += delta_time * theme::STRIDE_RATE;
            let bounce = stride.0.sin().abs() * theme::STRIDE_BOUNCE;
            sprite.size = Vec2::new(1.0 - bounce * 0.5, 1.0 + bounce);
        } else {
            stride.0 = 0.0;
            sprite.size = Vec2::new(1.0, 1.0);
        }

        if inside {
            transform.translation.y = theme::INDOOR_Y;
        } else if below {
            transform.translation.y = theme::HOLLOW_Y;
        } else if let Some(height) = height_at(&terrain, ground.x, ground.y) {
            let share = (theme::STEP_SMOOTHING * delta_time).clamp(0.0, 1.0);
            transform.translation.y += (height as f32 - transform.translation.y) * share;
        }
        feet = Some(transform.translation);
    });

    if let Some(feet) = feet {
        diorama.feet = feet;
    }
}
