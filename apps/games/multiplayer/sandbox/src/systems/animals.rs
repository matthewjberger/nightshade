use crate::components::{Arc, Pace, Ring};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

pub fn walk(time: Res<Time>, herd: Query<(&Ring, &Pace, &mut Arc, &mut LocalTransform)>) {
    let delta = time.delta_time;
    herd.for_each(|_entity, (ring, pace, arc, transform)| {
        arc.0 += pace.0 * delta;
        let x = arc.0.sin() * ring.0;
        let z = arc.0.cos() * ring.0;
        transform.translation = Vec3::new(x, 0.0, z);
        let heading = arc.0 + std::f32::consts::FRAC_PI_2;
        transform.rotation =
            nalgebra_glm::quat_angle_axis(heading + theme::MODEL_FACING, &Vec3::y());
    });
}
