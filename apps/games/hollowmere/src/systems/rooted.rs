use crate::components::{Regrow, Rooted};
use crate::queries::{flat, stand_at};
use crate::resources::VoxelWorld;
use nightshade::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn settle(
    world: Res<VoxelWorld>,
    mut props: Query<(&Rooted, Option<&Regrow>, &mut LocalTransform)>,
) {
    props.for_each_mut(|_entity, (rooted, regrow, transform)| {
        if regrow.is_some_and(|regrow| regrow.0 > 0.0) {
            return;
        }
        let from = transform.translation.y - rooted.0;
        if let Some(floor) = stand_at(&world, flat(transform.translation), from) {
            let wanted = floor + rooted.0;
            if (transform.translation.y - wanted).abs() > 0.001 {
                transform.translation.y = wanted;
            }
        }
    });
}
