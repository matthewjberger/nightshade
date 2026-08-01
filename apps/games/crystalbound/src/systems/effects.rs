use crate::components::{Home, Phase, Rise};
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn bob_the_crystal(time: Res<Time>, bobs: Query<(&Home, &Phase, &Rise, &mut LocalTransform)>) {
    let clock = time.uptime_seconds as f32;
    bobs.for_each(|_entity, (home, phase, rise, transform)| {
        transform.translation.y = home.0.y + (clock * 2.0 + phase.0).sin() * rise.0;
    });
}
