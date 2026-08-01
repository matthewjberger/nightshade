use crate::components::{Home, Phase};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn animate(time: Res<Time>, pickups: Query<(&Home, &Phase, &mut LocalTransform)>) {
    let clock = time.uptime_seconds as f32;
    pickups.for_each(|_entity, (home, phase, transform)| {
        let bob = (clock * 1.8 + phase.0).sin() * 0.08;
        transform.translation = Vec3::new(home.0.x, home.0.y + bob, home.0.z);
        transform.rotation = nalgebra_glm::quat_angle_axis(clock * 0.9 + phase.0, &Vec3::y());
    });
}
