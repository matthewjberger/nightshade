use crate::commands::populate_now;
use crate::components::{Home, Reach, Rest, Roll, Speed, Target};
use crate::queries::walkable;
use crate::resources::{Framing, Generation, Sheet, VoxelWorld};
use crate::theme;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;
use nightshade_sprite::prelude::*;
use nightshade_text::prelude::*;

pub fn populate(
    world: Res<VoxelWorld>,
    generation: Res<Generation>,
    sheet: Res<Sheet>,
    mut text_state: ResMut<TextState>,
    mut commands: Commands,
) {
    populate_now(&world, &generation, &sheet, &mut text_state, &mut commands);
}

pub type Walkers<'world, 'item> = nightshade::ecs::system_param::Query<
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

pub fn wander(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    framing: Res<Framing>,
    mut walkers: Walkers<'_, '_>,
) {
    let delta_time = time.delta_time.min(0.05);
    walkers.for_each_mut(
        |entity, (home, target, reach, speed, rest, roll, transform, sprite)| {
            if framing.subject == Some(entity) {
                return;
            }
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
                let here = walkable(&world, ground.x.floor() as i32, ground.y.floor() as i32);
                let there = walkable(&world, next.x.floor() as i32, next.y.floor() as i32);
                match (here, there) {
                    (Some(from), Some(to)) if (from - to).abs() <= 1 => {
                        transform.translation.x = next.x;
                        transform.translation.z = next.y;
                        if direction.x.abs() > 0.01 {
                            sprite.flip_x = direction.x < 0.0;
                        }
                    }
                    _ => {
                        target.0 = ground;
                        rest.0 = 0.5;
                    }
                }
            }
            if let Some(ground) = walkable(
                &world,
                transform.translation.x.floor() as i32,
                transform.translation.z.floor() as i32,
            ) {
                let wanted = ground as f32 + 1.0;
                let share = (theme::STEP_SMOOTHING * delta_time).clamp(0.0, 1.0);
                transform.translation.y += (wanted - transform.translation.y) * share;
            }
        },
    );
}
