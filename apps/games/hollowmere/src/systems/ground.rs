use crate::components::{Cooldowns, Heading, Stride};
use crate::queries::{column_of, column_settled, flat, stand_at};
use crate::resources::VoxelWorld;
use crate::theme;
use nightshade::prelude::*;
use nightshade_scene::prelude::LocalTransform;

type Figures<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (&'item Heading, &'item mut LocalTransform),
    (With<Stride>, Without<Cooldowns>),
>;

pub fn land(world: Res<VoxelWorld>, mut figures: Figures<'_, '_>) {
    figures.for_each_mut(|_entity, (_, transform)| {
        let spot = transform.translation;
        let (x, z) = (spot.x.floor() as i32, spot.z.floor() as i32);
        if !column_settled(&world, column_of(x, z)) {
            return;
        }
        let Some(floor) = stand_at(&world, flat(spot), spot.y) else {
            return;
        };
        if (spot.y - floor).abs() > theme::LANDING_SLACK {
            transform.translation.y = floor;
        }
    });
}
