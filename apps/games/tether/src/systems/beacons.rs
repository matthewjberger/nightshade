use crate::components::{Home, Phase};
use crate::data::{BEACON_BOB, BEACON_RATE};
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

pub fn bob_the_beacons(time: Res<Time>, beacons: Query<(&Home, &Phase, &mut LocalTransform)>) {
    let clock = time.uptime_seconds as f32;
    beacons.for_each(|_entity, (home, phase, transform)| {
        let rise = (clock * BEACON_RATE + phase.0).sin() * BEACON_BOB;
        transform.translation = Vec3::new(home.0.x, home.0.y + rise, home.0.z);
    });
}
