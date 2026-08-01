use crate::components::{Rest, Roam, Speed, Target};
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn wander(
    time: Res<Time>,
    animals: Query<(
        &mut Target,
        &Speed,
        &mut Rest,
        &mut Roam,
        &mut LocalTransform,
        &mut Sprite,
    )>,
) {
    let delta_time = time.delta_time;
    animals.for_each(|_entity, (target, speed, rest, roam, transform, sprite)| {
        let position = Vec2::new(transform.translation.x, transform.translation.y);
        let to_target = target.0 - position;
        let distance = to_target.magnitude();

        if distance < 0.05 {
            rest.0 -= delta_time;
            if rest.0 <= 0.0 {
                let turn = roam.0.f32() * std::f32::consts::TAU;
                let reach = 1.5 + roam.0.f32() * 3.0;
                let candidate = position + Vec2::new(turn.cos(), turn.sin()) * reach;
                target.0 = Vec2::new(
                    candidate
                        .x
                        .clamp(theme::GROUND_ORIGIN.x + 2.0, -theme::GROUND_ORIGIN.x - 2.0),
                    candidate
                        .y
                        .clamp(theme::GROUND_ORIGIN.y + 2.0, -theme::GROUND_ORIGIN.y - 3.0),
                );
                rest.0 = 1.0 + roam.0.f32() * 3.0;
            }
            return;
        }

        let step = (speed.0 * delta_time).min(distance);
        let direction = to_target / distance;
        transform.translation.x += direction.x * step;
        transform.translation.y += direction.y * step;
        if direction.x.abs() > 0.01 {
            sprite.flip_x = direction.x < 0.0;
        }
    });
}
