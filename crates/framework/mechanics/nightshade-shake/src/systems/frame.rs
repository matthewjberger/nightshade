use crate::components::{ShakeOffset, ShakeTurn, Trauma};
use crate::queries::weight;
use crate::resources::ShakeSettings;
use nalgebra_glm::vec3;
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;
use nightshade_random::Random;

pub fn settle_the_shake(
    time: Res<Time>,
    settings: Res<ShakeSettings>,
    mut random: ResMut<Random>,
    shaken: Query<(&mut Trauma, &mut ShakeOffset, &mut ShakeTurn)>,
) {
    let delta = time.delta_time;
    shaken.for_each(|_entity, (trauma, offset, turn)| {
        trauma.0 = (trauma.0 - settings.decay * delta).max(0.0);
        let amount = weight(trauma);
        offset.0 = vec3(
            nightshade_random::commands::span(&mut random, settings.reach) * amount,
            nightshade_random::commands::span(&mut random, settings.reach) * amount,
            0.0,
        );
        turn.0 = nightshade_random::commands::span(&mut random, settings.turn) * amount;
    });
}
