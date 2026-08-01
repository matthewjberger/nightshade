use crate::components::{HalfExtent, Speed};
use crate::data;
use crate::data::Ask;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn drive(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    paddles: Query<(&Speed, &HalfExtent, &mut LocalTransform)>,
) {
    let delta_time = time.delta_time;
    let direction = actions.axis(Ask::Steer);
    paddles.for_each(|_entity, (speed, half_extent, transform)| {
        let limit = data::INNER_HALF_WIDTH - half_extent.0.x;
        transform.translation.x =
            (transform.translation.x + direction * speed.0 * delta_time).clamp(-limit, limit);
    });
}
