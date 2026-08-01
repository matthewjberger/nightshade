use crate::components::{
    Combo, Facing, Fling, Health, Kind, Pose, Rest, Slot, Speed, Stride, Strike, Stun,
};
use crate::data;
use crate::queries::walk_pose;
use crate::resources::Ring;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn approach(
    time: Res<Time>,
    ring: Res<Ring>,
    walkers: Pack,
    healths: Query<(&Health,), Without<Combo>>,
) {
    let delta_time = if ring.hit_stop > 0.0 {
        0.0
    } else {
        time.delta_time
    };
    let mut down = Vec::new();
    healths.for_each(|entity, (health,)| {
        if health.0 <= 0.0 {
            down.push(entity);
        }
    });
    let target = ring.feet;
    walkers.for_each(
        |entity, (speed, slot, facing, stride, pose, stun, strike, transform)| {
            if down.contains(&entity) || stun.0 > 0.0 || strike.0 > 0.0 {
                return;
            }
            let lane = slot.0 * data::THUG_SPREAD;
            let toward = Vec2::new(
                target.x - transform.translation.x,
                target.z + lane - transform.translation.z,
            );
            if toward.x != 0.0 {
                facing.0 = toward.x.signum();
            }
            let stand = data::REACH * data::THUG_APPROACH + slot.0.abs() * data::THUG_STAGGER;
            if toward.x.abs() < stand && toward.y.abs() < data::LANE_REACH * 0.8 {
                stride.0 = 0.0;
                if pose.0 != data::THUG_BLOW && pose.0 != data::THUG_WIND {
                    pose.0 = data::POSE_IDLE;
                }
                return;
            }
            let step = toward.normalize() * speed.0 * delta_time;
            transform.translation.x += step.x;
            transform.translation.z =
                (transform.translation.z + step.y).clamp(-data::LANE_HALF, data::LANE_HALF);
            stride.0 += delta_time * data::STRIDE_RATE;
            pose.0 = walk_pose(stride.0);
        },
    );
}

pub fn swing(time: Res<Time>, mut ring: ResMut<Ring>, mut fighters: Fists) {
    let delta_time = if ring.hit_stop > 0.0 {
        0.0
    } else {
        time.delta_time
    };
    let target = ring.feet;
    let mut spots: Vec<(Entity, Vec3)> = Vec::new();
    fighters
        .spots
        .for_each_mut(|entity, (transform,)| spots.push((entity, transform.translation)));

    let mut damage = 0.0;
    let mut push = 0.0;
    fighters.act.for_each_mut(
        |entity, (rest, strike, pose, fling, kind, facing, stun, health)| {
            rest.0 = (rest.0 - delta_time).max(0.0);
            if health.0 <= 0.0 {
                return;
            }
            let Some((_, at)) = spots.iter().copied().find(|(other, _)| *other == entity) else {
                return;
            };
            let plan = &data::THUG_KINDS[kind.0 as usize];
            if stun.0 > 0.0 {
                strike.0 = 0.0;
                return;
            }
            if strike.0 > 0.0 {
                let before = strike.0;
                strike.0 = (strike.0 - delta_time).max(0.0);
                if strike.0 > data::BLOW_TIME {
                    pose.0 = data::THUG_WIND;
                    return;
                }
                pose.0 = data::THUG_BLOW;
                let along = (target.x - at.x) * facing.0;
                let across = (target.z - at.z).abs();
                if before > data::BLOW_MOMENT && strike.0 <= data::BLOW_MOMENT {
                    fling.0 += facing.0 * data::THUG_LUNGE;
                    if (-0.3..data::REACH + plan.reach).contains(&along)
                        && across < data::LANE_REACH
                    {
                        damage += plan.damage;
                        push = facing.0 * data::THUG_PUSH;
                    }
                }
                if strike.0 <= 0.0 {
                    pose.0 = data::POSE_IDLE;
                }
                return;
            }
            if rest.0 > 0.0 {
                return;
            }
            let reach = data::REACH * data::THUG_APPROACH + plan.reach + 0.2;
            if (target.x - at.x).abs() > reach || (target.z - at.z).abs() > data::LANE_REACH {
                return;
            }
            rest.0 = plan.rest;
            strike.0 = data::BLOW_TIME + data::THUG_WIND_TIME;
            pose.0 = data::THUG_WIND;
        },
    );
    ring.thug_damage += damage;
    if push != 0.0 {
        ring.thug_push = push;
    }
}

pub type ThugAct<'world, 'item> = Query<
    'world,
    (
        &'item mut Rest,
        &'item mut Strike,
        &'item mut Pose,
        &'item mut Fling,
        &'item Kind,
        &'item Facing,
        &'item Stun,
        &'item Health,
    ),
    Without<Combo>,
>;

pub type ThugSpots<'world, 'item> =
    Query<'world, (&'item LocalTransform,), (Without<Combo>, With<Rest>)>;

nightshade::ecs::query_bundle! {
    pub struct Fists<'world, 'item> {
        pub spots: ThugSpots<'world, 'item>,
        pub act: ThugAct<'world, 'item>,
    }
}

pub type Pack<'world, 'item> = Query<
    'world,
    (
        &'item Speed,
        &'item Slot,
        &'item mut Facing,
        &'item mut Stride,
        &'item mut Pose,
        &'item Stun,
        &'item Strike,
        &'item mut LocalTransform,
    ),
    Without<Combo>,
>;
