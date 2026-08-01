use crate::components::{Drift, Orbit, Phase, Spin};
use crate::data::CLOUD_REACH;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn drift(time: Res<Time>, mut clouds: Query<(&Drift, &mut LocalTransform), Without<Orbit>>) {
    let delta_time = time.delta_time;
    clouds.for_each_mut(|_entity, (drift, transform)| {
        transform.translation += drift.0 * delta_time;
        if transform.translation.x > CLOUD_REACH {
            transform.translation.x -= CLOUD_REACH * 2.0;
        }
        if transform.translation.z > CLOUD_REACH {
            transform.translation.z -= CLOUD_REACH * 2.0;
        }
    });
}

pub fn wheel(
    time: Res<Time>,
    mut birds: Query<(&Orbit, &Spin, &Drift, &mut Phase, &mut LocalTransform)>,
) {
    let delta_time = time.delta_time;
    birds.for_each_mut(|_entity, (orbit, spin, drift, phase, transform)| {
        phase.0 += spin.0 * delta_time;
        let angle = phase.0;
        let radius_x = drift.0.x;
        let radius_z = drift.0.z;
        let position = Vec3::new(
            orbit.0.x + angle.cos() * radius_x,
            orbit.0.y + (angle * 3.0).sin() * 1.6,
            orbit.0.z + angle.sin() * radius_z,
        );
        let forward = Vec3::new(-angle.sin() * radius_x, 0.0, angle.cos() * radius_z) * spin.0;
        let yaw = forward.x.atan2(forward.z);
        let flap = (angle * 14.0).sin() * 0.35;
        transform.translation = position;
        transform.rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y())
            * nalgebra_glm::quat_angle_axis(flap, &Vec3::z());
    });
}
