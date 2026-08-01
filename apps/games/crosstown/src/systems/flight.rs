use crate::components::{Fling, Lift, Rise};
use crate::data;
use crate::resources::Ring;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn fall(
    time: Res<Time>,
    ring: Res<Ring>,
    bodies: Query<(&mut Lift, &mut Rise, &mut Fling, &mut LocalTransform)>,
) {
    let delta_time = if ring.hit_stop > 0.0 {
        0.0
    } else {
        time.delta_time
    };
    bodies.for_each(|_entity, (lift, rise, fling, transform)| {
        let airborne = lift.0 > 0.0 || rise.0 > 0.0;
        if !airborne && fling.0 == 0.0 {
            return;
        }
        if airborne {
            rise.0 -= data::GRAVITY * delta_time;
            lift.0 += rise.0 * delta_time;
            if lift.0 <= 0.0 {
                lift.0 = 0.0;
                rise.0 = 0.0;
                fling.0 *= data::LAND_DRAG;
            }
        }
        if fling.0 == 0.0 {
            return;
        }
        transform.translation.x += fling.0 * delta_time;
        let drag = if lift.0 > 0.0 {
            data::AIR_DRAG
        } else {
            data::GROUND_DRAG
        };
        fling.0 *= drag.powf(delta_time * 60.0);
        if fling.0.abs() < 0.08 {
            fling.0 = 0.0;
        }
    });
}

pub fn tumble(shards: Query<(&Lift, &mut LocalTransform), With<Lifetime>>) {
    shards.for_each(|_entity, (lift, transform)| {
        if transform.translation.y != lift.0 {
            transform.translation.y = lift.0;
        }
    });
}
