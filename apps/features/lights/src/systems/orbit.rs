use crate::components::Orbit;
use crate::queries::light_position;
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_platform::prelude::*;

pub fn carry_the_lights(time: Res<Time>, mut orbits: Query<(&Orbit, &mut LocalTransform)>) {
    let seconds = time.uptime_milliseconds as f32 / 1000.0;
    orbits.for_each_mut(|_entity, (orbit, transform)| {
        transform.translation = light_position(orbit.0, seconds);
    });
}
