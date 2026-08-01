use crate::commands::{Walk, walk_toward};
use crate::components::{Beat, Heading, Pause, Post, Speed, Stride};
use crate::data::{DOG_ROUTES, GUARD_ROUTES, TOWN};
use crate::queries::{guessed_ground, spot_of};
use crate::resources::{Generation, Places, VoxelWorld};
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

type Walkers<'world, 'item> = nightshade::ecs::system_param::Query<
    'world,
    (
        &'item Beat,
        &'item Speed,
        &'item mut Post,
        &'item mut Pause,
        &'item mut Heading,
        &'item mut Stride,
        &'item mut LocalTransform,
    ),
>;

fn route(beat: usize) -> &'static [Vec2] {
    if beat < GUARD_ROUTES.len() {
        GUARD_ROUTES[beat]
    } else {
        DOG_ROUTES[beat - GUARD_ROUTES.len()]
    }
}

pub fn patrol(
    time: Res<Time>,
    world: Res<VoxelWorld>,
    generation: Res<Generation>,
    places: Res<Places>,
    mut walkers: Walkers<'_, '_>,
) {
    let delta_time = time.delta_time;
    walkers.for_each_mut(
        |_entity, (beat, speed, post, pause, heading, stride, transform)| {
            if pause.0 > 0.0 {
                pause.0 -= delta_time;
                return;
            }
            let path = route(beat.0);
            let goal = guessed_ground(
                &generation,
                spot_of(&places, TOWN, path[post.0 % path.len()]),
            );
            let walk = walk_toward(
                &world, transform, heading, stride, goal, speed.0, delta_time,
            );
            if walk != Walk::Moved {
                post.0 = (post.0 + 1) % path.len();
                pause.0 = if beat.0 < GUARD_ROUTES.len() {
                    theme::GUARD_PAUSE_SECONDS
                } else {
                    theme::GUARD_PAUSE_SECONDS * 0.08
                };
            }
        },
    );
}
