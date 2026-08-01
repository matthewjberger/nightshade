use crate::components::{FollowOffset, Following};
use crate::queries::snapped;
use crate::resources::FollowSettings;
use nalgebra_glm::{Quat, Vec3, quat_angle_axis, vec3};
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::Res;
use nightshade_ease::share;
use nightshade_platform::contract::time::Time;
use nightshade_scene::contract::LocalTransform;

pub fn follow_the_target(time: Res<Time>, settings: Res<FollowSettings>, storage: &mut Storage) {
    let delta = time.delta_time;
    let rate = settings.rate;
    let snap = settings.snap;
    let aim = settings.aim;
    let mut wanted = Vec::new();
    for (entity, (transform, following, offset)) in storage
        .query_ref::<(&LocalTransform, &Following, &FollowOffset)>()
        .iter()
    {
        let Some(target) = storage.get::<LocalTransform>(following.0) else {
            continue;
        };
        wanted.push((
            entity,
            transform.translation,
            target.translation + offset.0,
            target.translation,
        ));
    }
    let amount = share(rate, delta);
    for (entity, here, there, looked_at) in wanted {
        let Some(transform) = storage.get_mut::<LocalTransform>(entity) else {
            continue;
        };
        let mut place = here + (there - here) * amount;
        if let Some(step) = snap {
            place.x = snapped(place.x, step);
            place.y = snapped(place.y, step);
        }
        transform.translation = place;
        if aim {
            transform.rotation = aimed_at(place, looked_at);
        }
    }
}

pub fn aimed_at(from: Vec3, to: Vec3) -> Quat {
    let forward = to - from;
    let flat = (forward.x * forward.x + forward.z * forward.z).sqrt();
    if forward.norm() <= f32::EPSILON {
        return Quat::identity();
    }
    let yaw = (-forward.x).atan2(-forward.z);
    let pitch = forward.y.atan2(flat.max(f32::EPSILON));
    quat_angle_axis(yaw, &vec3(0.0, 1.0, 0.0)) * quat_angle_axis(pitch, &vec3(1.0, 0.0, 0.0))
}
