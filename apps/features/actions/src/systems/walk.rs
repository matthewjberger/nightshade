use crate::data::{ARENA_EXTENT, Ask, DASH_MULTIPLIER, GRAVITY, JUMP_SPEED, WALK_SPEED};
use crate::resources::ActionsResources;
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::Time;
use nightshade_scene::contract::LocalTransform;

pub fn walk_the_cube(
    actions: Res<Actions<Ask>>,
    time: Res<Time>,
    mut resources: ResMut<ActionsResources>,
    mut cubes: Query<(&mut LocalTransform,)>,
) {
    let delta_time = time.delta_time;
    let walk = actions.axis2(Ask::Walk);
    let speed = if actions.pressed(Ask::Dash) {
        WALK_SPEED * DASH_MULTIPLIER
    } else {
        WALK_SPEED
    };
    if actions.just_pressed(Ask::Jump) && resources.rise <= 0.0 {
        resources.lift = JUMP_SPEED;
    }
    resources.lift -= GRAVITY * delta_time;
    resources.rise = (resources.rise + resources.lift * delta_time).max(0.0);
    if resources.rise <= 0.0 && resources.lift < 0.0 {
        resources.lift = 0.0;
    }
    let rise = resources.rise;
    cubes.for_entity(resources.cube, |(transform,)| {
        transform.translation.x = (transform.translation.x + walk.x * speed * delta_time)
            .clamp(-ARENA_EXTENT, ARENA_EXTENT);
        transform.translation.z = (transform.translation.z - walk.y * speed * delta_time)
            .clamp(-ARENA_EXTENT, ARENA_EXTENT);
        transform.translation.y = 0.5 + rise;
    });
}
