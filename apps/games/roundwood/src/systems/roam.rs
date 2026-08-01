use crate::components::{Base, Home, Reach, Rest, Roll, Speed, Spot, Stride, Target};
use crate::data;
use crate::queries::cylinder;
use crate::queries::town::can_step;
use crate::resources::Town;
use nalgebra_glm::Vec2;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_sprite::prelude::*;

pub type Plans<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Home,
        &'item mut Target,
        &'item Reach,
        &'item mut Rest,
        &'item mut Roll,
    ),
>;

pub type Bodies<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item mut Spot,
        &'item Base,
        &'item Speed,
        &'item mut Stride,
        &'item mut Sprite,
    ),
>;

nightshade::ecs::query_bundle! {
    pub struct Wanderers<'world, 'item> {
        pub plans: Plans<'world, 'item>,
        pub bodies: Bodies<'world, 'item>,
    }
}

pub fn wander(time: Res<Time>, town: Res<Town>, mut walkers: Wanderers<'_, '_>) {
    let delta_time = time.delta_time;

    let mut held: Vec<(Entity, Vec2, f32)> = Vec::new();
    walkers
        .plans
        .for_each_mut(|entity, (_, target, _, rest, _)| {
            held.push((entity, target.0, rest.0));
        });

    let mut outcomes: Vec<(Entity, bool, f32)> = Vec::new();
    for (entity, target, rest) in held {
        let mut choose = false;
        let mut remaining = rest;
        walkers
            .bodies
            .for_entity(entity, |(spot, base, speed, stride, sprite)| {
                if rest > 0.0 {
                    remaining = rest - delta_time;
                    stride.0 = 0.0;
                    sprite.size = base.0;
                    return;
                }
                let to_target = cylinder::town_delta(spot.0, target);
                if to_target.magnitude() < data::ARRIVE_RANGE {
                    choose = true;
                    return;
                }
                let step = to_target.normalize() * speed.0 * delta_time;
                let mut moved = false;
                let across = Vec2::new(spot.0.x + step.x, spot.0.y);
                if can_step(&town, spot.0, across) {
                    spot.0.x = across.x;
                    moved = true;
                }
                let along = Vec2::new(spot.0.x, spot.0.y + step.y);
                if can_step(&town, spot.0, along) {
                    spot.0.y = cylinder::wrap(along.y);
                    moved = true;
                }
                if !moved {
                    choose = true;
                    return;
                }
                if step.x.abs() > 0.0001 {
                    sprite.flip_x = step.x < 0.0;
                }
                stride.0 += delta_time * data::STRIDE_RATE * 0.6;
                let bounce = stride.0.sin().abs() * data::STRIDE_BOUNCE * 0.7;
                sprite.size = Vec2::new(base.0.x * (1.0 - bounce * 0.5), base.0.y * (1.0 + bounce));
            });
        outcomes.push((entity, choose, remaining));
    }

    for (entity, choose, remaining) in outcomes {
        walkers
            .plans
            .for_entity(entity, |(home, target, reach, rest, roll)| {
                if choose {
                    rest_and_choose(home, target, reach, rest, roll);
                } else {
                    rest.0 = remaining;
                }
            });
    }
}

fn rest_and_choose(
    home: &Home,
    target: &mut Target,
    reach: &Reach,
    rest: &mut Rest,
    roll: &mut Roll,
) {
    let span = reach.0;
    let across = (home.0.x + (roll.0.f32() - 0.5) * 2.0 * span).clamp(
        data::WEST_BEACH_END as f32 + 0.5,
        data::EAST_BEACH_START as f32 - 0.5,
    );
    let along = cylinder::wrap(home.0.y + (roll.0.f32() - 0.5) * 2.0 * span);
    target.0 = Vec2::new(across, along);
    rest.0 = data::NEIGHBOR_REST * (0.5 + roll.0.f32());
}
