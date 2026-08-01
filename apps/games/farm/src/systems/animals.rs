use crate::components::{Home, Reach, Rest, Roll, Speed, Target};
use crate::queries::camera::screen_right;
use crate::queries::terrain::{can_step, height_at};
use crate::resources::{Rig, Terrain};
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub type Walkers<'world, 'item> = Query<
    'world,
    (
        &'item Home,
        &'item mut Target,
        &'item Reach,
        &'item Speed,
        &'item mut Rest,
        &'item mut Roll,
        &'item mut LocalTransform,
        &'item mut Sprite,
    ),
>;

pub fn wander(time: Res<Time>, terrain: Res<Terrain>, rig: Res<Rig>, walkers: Walkers<'_, '_>) {
    let delta_time = time.delta_time;
    let right = screen_right(rig.yaw);
    walkers.for_each(
        |_entity, (home, target, reach, speed, rest, roll, transform, sprite)| {
            let ground = Vec2::new(transform.translation.x, transform.translation.z);
            let to_target = target.0 - ground;
            let distance = to_target.magnitude();

            if distance < 0.05 {
                rest.0 -= delta_time;
                if rest.0 <= 0.0 {
                    let turn = roll.0.f32() * std::f32::consts::TAU;
                    let span = roll.0.f32() * reach.0;
                    target.0 = home.0 + Vec2::new(turn.cos(), turn.sin()) * span;
                    rest.0 = 1.0 + roll.0.f32() * 3.0;
                }
            } else {
                let step = (speed.0 * delta_time).min(distance);
                let direction = to_target / distance;
                let next = ground + direction * step;
                if can_step(&terrain, ground, next) {
                    transform.translation.x = next.x;
                    transform.translation.z = next.y;
                    let sideways = direction.dot(&right);
                    if sideways.abs() > 0.01 {
                        sprite.flip_x = sideways < 0.0;
                    }
                } else {
                    target.0 = ground;
                    rest.0 = 0.5;
                }
            }

            if let Some(height) =
                height_at(&terrain, transform.translation.x, transform.translation.z)
            {
                let share = (theme::STEP_SMOOTHING * delta_time).clamp(0.0, 1.0);
                transform.translation.y += (height as f32 - transform.translation.y) * share;
            }
        },
    );
}
