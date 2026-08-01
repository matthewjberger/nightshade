use crate::commands::walk_toward;
use crate::components::{Beat, Heading, Pause, Post, Speed, Stride};
use crate::data::{DOG_ROUTES, GUARD_PAUSE_SECONDS, GUARD_ROUTES};
use crate::queries::ground;
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

pub fn patrol(time: Res<Time>, mut walkers: Walkers<'_, '_>) {
    let delta_time = time.delta_time;
    walkers.for_each_mut(
        |_entity, (beat, speed, post, pause, heading, stride, transform)| {
            if pause.0 > 0.0 {
                pause.0 -= delta_time;
                return;
            }
            let path = route(beat.0);
            let goal = ground(path[post.0 % path.len()]);
            if walk_toward(transform, heading, stride, goal, speed.0, delta_time) {
                post.0 = (post.0 + 1) % path.len();
                pause.0 = if beat.0 < GUARD_ROUTES.len() {
                    GUARD_PAUSE_SECONDS
                } else {
                    GUARD_PAUSE_SECONDS * 0.08
                };
            }
        },
    );
}
