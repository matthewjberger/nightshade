use crate::components::{Speed, Target};
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn walk_the_farmer(
    time: Res<Time>,
    input: Res<Input>,
    farmers: Query<
        (
            &Speed,
            &mut LocalTransform,
            &mut Sprite,
            &mut SpriteAnimator,
        ),
        Without<Target>,
    >,
) {
    let mut direction = Vec2::zeros();
    if input.keyboard.is_key_pressed(KeyCode::KeyA)
        || input.keyboard.is_key_pressed(KeyCode::ArrowLeft)
    {
        direction.x -= 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::KeyD)
        || input.keyboard.is_key_pressed(KeyCode::ArrowRight)
    {
        direction.x += 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::KeyW)
        || input.keyboard.is_key_pressed(KeyCode::ArrowUp)
    {
        direction.y += 1.0;
    }
    if input.keyboard.is_key_pressed(KeyCode::KeyS)
        || input.keyboard.is_key_pressed(KeyCode::ArrowDown)
    {
        direction.y -= 1.0;
    }
    let moving = direction.magnitude_squared() > 0.0;
    if moving {
        direction = direction.normalize();
    }
    let delta_time = time.delta_time;

    farmers.for_each(|_entity, (speed, transform, sprite, walk)| {
        if moving {
            let step = direction * speed.0 * delta_time;
            transform.translation.x = (transform.translation.x + step.x)
                .clamp(theme::GROUND_ORIGIN.x + 1.0, -theme::GROUND_ORIGIN.x - 1.0);
            transform.translation.y = (transform.translation.y + step.y)
                .clamp(theme::GROUND_ORIGIN.y + 1.0, -theme::GROUND_ORIGIN.y - 2.0);
            if direction.x != 0.0 {
                sprite.flip_x = direction.x < 0.0;
            }
            if !walk.playing {
                walk.play();
            }
        } else if walk.playing {
            walk.pause();
            walk.reset();
            if let Some(region) = walk.current_region() {
                sprite.region = region;
            }
        }
    });
}
