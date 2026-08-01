use crate::components::{Airborne, Coyote, JumpAsked};
use crate::resources::PlatformerSettings;
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;

pub fn spend_the_mercy(
    time: Res<Time>,
    settings: Res<PlatformerSettings>,
    bodies: Query<(&Airborne, &mut Coyote, &mut JumpAsked)>,
) {
    let delta = time.delta_time;
    bodies.for_each(|_entity, (airborne, coyote, asked)| {
        coyote.0 = if airborne.0 {
            (coyote.0 - delta).max(0.0)
        } else {
            settings.coyote
        };
        asked.0 = (asked.0 - delta).max(0.0);
    });
}
