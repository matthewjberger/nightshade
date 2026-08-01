use crate::components::Spin;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn turn_the_cube(time: Res<Time>, spinners: Query<(&Spin, &mut LocalTransform)>) {
    let delta_time = time.delta_time;
    let axis = nalgebra_glm::normalize(&Vec3::new(1.0, 1.0, 0.35));
    spinners.for_each(|_entity, (spin, transform)| {
        let rotation = nalgebra_glm::quat_angle_axis(spin.0 * delta_time, &axis);
        transform.rotation = rotation * transform.rotation;
    });
}
