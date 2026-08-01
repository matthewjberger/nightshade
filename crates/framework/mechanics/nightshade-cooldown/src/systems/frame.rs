use crate::components::Cooldown;
use nightshade::prelude::*;
use nightshade_platform::contract::time::Time;

pub fn spend_the_cooldowns(time: Res<Time>, waiting: Query<(&mut Cooldown,)>) {
    let delta = time.delta_time;
    waiting.for_each(|_entity, (cooldown,)| {
        if cooldown.0 > 0.0 {
            cooldown.0 = (cooldown.0 - delta).max(0.0);
        }
    });
}
