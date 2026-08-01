use crate::components::Spin;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;

pub fn tumble(time: Res<Time>, mut stones: Query<(&Spin, &mut LocalTransform)>) {
    let delta_time = time.delta_time;
    stones.for_each_mut(|_entity, (stone, transform)| {
        let step = nalgebra_glm::quat_angle_axis(stone.0 * delta_time, &Vec3::y());
        transform.rotation = (step * transform.rotation).normalize();
    });
}
