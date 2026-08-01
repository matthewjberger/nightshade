use crate::contract::Lifetime;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;

pub fn expire_lifetimes_system(
    time: Res<Time>,
    mut commands: Commands,
    lifetimes: Query<(&mut Lifetime,)>,
) {
    let delta_time = time.delta_time;
    let mut expired = Vec::new();
    lifetimes.for_each(|entity, (lifetime,)| {
        lifetime.remaining -= delta_time;
        if lifetime.remaining <= 0.0 {
            expired.push(entity);
        }
    });
    for entity in expired {
        commands.despawn_recursive(entity);
    }
}
