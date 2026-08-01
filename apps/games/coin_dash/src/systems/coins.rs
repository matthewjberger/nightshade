use crate::components::{Home, Phase};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn animate(time: Res<Time>, coins: Query<(&Home, &Phase, &mut LocalTransform)>) {
    let clock = time.uptime_seconds as f32;
    let delta_time = time.delta_time;
    coins.for_each(|_entity, (home, phase, transform)| {
        let bob = (clock * 1.5 + phase.0).sin() * 0.2;
        transform.translation = Vec3::new(home.0.x, home.0.y + bob, home.0.z);
        transform.rotation =
            nalgebra_glm::quat_angle_axis(delta_time * 3.0, &Vec3::y()) * transform.rotation;
    });
}
