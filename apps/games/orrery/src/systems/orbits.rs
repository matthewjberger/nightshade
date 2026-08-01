use crate::components::Body;
use crate::queries;
use crate::resources::OrreryResources;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;

pub fn advance(
    time: Res<Time>,
    mut resources: ResMut<OrreryResources>,
    mut bodies: Query<(&Body, &mut LocalTransform)>,
) {
    resources.flight.elapsed += time.delta_time;
    let elapsed = resources.flight.elapsed;
    bodies.for_each_mut(|_entity, (body, transform)| {
        transform.translation = queries::body_position(body.0, elapsed);
    });
}
