use crate::components::{Bob, Perch, Phase, Spin};
use crate::queries::lift;
use crate::resources::HoverSettings;
use nalgebra_glm::{Vec3, quat_angle_axis};
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;
use nightshade_scene::contract::LocalTransform;

pub fn hold_them_up(
    time: Res<Time>,
    settings: Res<HoverSettings>,
    things: Query<(&Perch, &Bob, &Phase, &Spin, &mut LocalTransform)>,
) {
    let clock = time.uptime_seconds as f32;
    let delta = time.delta_time;
    things.for_each(|_entity, (perch, bob, phase, spin, transform)| {
        let up = lift(bob, phase, &settings, clock);
        transform.translation = Vec3::new(perch.0.x, perch.0.y + up, perch.0.z);
        if spin.0 != 0.0 {
            transform.rotation = quat_angle_axis(delta * spin.0, &Vec3::y()) * transform.rotation;
        }
    });
}
