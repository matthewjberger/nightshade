use crate::components::{Grow, HalfExtent, Pop};
use crate::queries;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn animate(
    time: Res<Time>,
    mut commands: Commands,
    mut popping: Query<(&HalfExtent, &mut Pop, &mut LocalTransform)>,
    growing: Query<(&HalfExtent, &mut Grow, &mut LocalTransform)>,
) {
    let delta_time = time.delta_time;

    let mut spent = Vec::new();
    popping.for_each_mut(|entity, (half_extent, remaining, transform)| {
        remaining.0 -= delta_time;
        if remaining.0 <= 0.0 {
            spent.push(entity);
            return;
        }
        let share = queries::pop_scale(remaining.0);
        transform.scale = Vec3::new(
            half_extent.0.x * 2.0 * share,
            half_extent.0.y * 2.0 * share,
            share.max(0.05),
        );
    });
    for entity in spent {
        commands.despawn(entity);
    }

    let mut ready = Vec::new();
    growing.for_each(|entity, (half_extent, remaining, transform)| {
        remaining.0 -= delta_time;
        let share = queries::grow_scale(remaining.0);
        transform.scale = Vec3::new(
            half_extent.0.x * 2.0 * share,
            half_extent.0.y * 2.0 * share,
            1.0,
        );
        if remaining.0 <= 0.0 {
            ready.push(entity);
        }
    });
    for entity in ready {
        commands.remove::<(Grow,)>(entity);
    }
}
