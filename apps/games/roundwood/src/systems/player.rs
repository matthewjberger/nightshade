use crate::components::{Base, Lift, Speed, Spot, Stride};
use crate::data;
use crate::data::Ask;
use crate::queries::cylinder;
use crate::queries::town::can_step;
use crate::resources::{Chat, Town, Village};
use crate::tags::Player;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_sprite::prelude::*;

pub type Players<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Spot,
        &'item Base,
        &'item Speed,
        &'item Lift,
        &'item mut Stride,
        &'item mut Sprite,
    ),
    WithTag<Player>,
>;

pub fn walk_the_neighbor(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    town: Res<Town>,
    chat: Res<Chat>,
    mut village: ResMut<Village>,
    mut players: Players<'_, '_>,
) {
    let mut direction = Vec2::zeros();
    if !chat.shown {
        let walk = actions.axis2(Ask::Walk);
        direction = Vec2::new(walk.x, -walk.y);
    }
    let moving = direction.magnitude_squared() > 0.0;
    let delta_time = time.delta_time;
    let mut feet = None;

    players.for_each_mut(|_entity, (spot, base, speed, lift, stride, sprite)| {
        if moving {
            let step = direction.normalize() * speed.0 * delta_time;
            let across = Vec2::new(spot.0.x + step.x, spot.0.y);
            if can_step(&town, spot.0, across) {
                spot.0.x = across.x;
            }
            let along = Vec2::new(spot.0.x, spot.0.y + step.y);
            if can_step(&town, spot.0, along) {
                spot.0.y = cylinder::wrap(along.y);
            }
            if direction.x != 0.0 {
                sprite.flip_x = direction.x < 0.0;
            }
            stride.0 += delta_time * data::STRIDE_RATE;
            let bounce = stride.0.sin().abs() * data::STRIDE_BOUNCE;
            sprite.size = Vec2::new(base.0.x * (1.0 - bounce * 0.5), base.0.y * (1.0 + bounce));
        } else {
            stride.0 = 0.0;
            sprite.size = base.0;
        }
        feet = Some((spot.0, lift.0));
    });

    if let Some((town_spot, lift)) = feet {
        village.feet = town_spot;
        village.lift = lift;
    }
}
