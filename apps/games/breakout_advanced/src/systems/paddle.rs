use crate::components::{HalfExtent, Lean, LeanRate, Speed};
use crate::data;
use crate::data::Ask;
use crate::resources::BreakoutAdvancedResources;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn drive(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    resources: Res<BreakoutAdvancedResources>,
    paddles: Query<(
        &Speed,
        &HalfExtent,
        &mut Lean,
        &mut LeanRate,
        &mut LocalTransform,
    )>,
) {
    let delta_time = time.delta_time * resources.juice.time_scale;
    let direction = actions.axis(Ask::Steer);

    paddles.for_each(|_entity, (speed, half_extent, lean, rate, transform)| {
        let limit = data::INNER_HALF_WIDTH - half_extent.0.x;
        let before = transform.translation.x;
        let after = (before + direction * speed.0 * delta_time).clamp(-limit, limit);
        transform.translation.x = after;
        transform.translation.y = data::PADDLE_Y;

        let travelled = if delta_time > f32::EPSILON {
            (after - before) / delta_time
        } else {
            0.0
        };
        let target = (travelled * data::LEAN_GAIN).clamp(-data::LEAN_LIMIT, data::LEAN_LIMIT);
        rate.0 += (target - lean.0) * data::LEAN_STIFFNESS * delta_time;
        rate.0 -= rate.0 * data::LEAN_DAMPING * delta_time;
        lean.0 += rate.0 * delta_time;

        let bend = lean.0.abs() * data::PADDLE_SQUASH_GAIN;
        transform.rotation = nalgebra_glm::quat_angle_axis(-lean.0, &Vec3::z());
        transform.scale = Vec3::new(
            half_extent.0.x * 2.0 * (1.0 + bend),
            half_extent.0.y * 2.0 * (1.0 - bend * 0.6).max(0.25),
            1.0,
        );
    });
}
