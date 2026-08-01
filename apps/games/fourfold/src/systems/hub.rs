use crate::commands as game;
use crate::components::Extent;
use crate::contract::{Cue, Dimension};
use crate::data;
use crate::queries;
use crate::resources::FourfoldResources;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn look(mut resources: ResMut<FourfoldResources>) {
    let turn = resources.drive.look;
    let walker = &mut resources.walker;
    walker.yaw -= turn.x;
    walker.pitch = (walker.pitch - turn.y).clamp(-data::PITCH_LIMIT, data::PITCH_LIMIT);
}

pub fn walk(
    time: Res<Time>,
    mut resources: ResMut<FourfoldResources>,
    blocks: Query<(&Dimension, &Extent, &LocalTransform)>,
) {
    let delta_time = time.delta_time.min(0.05);
    let solids = queries::gather_solids(blocks, Dimension::Hub);

    let asked = resources.drive.wish;
    let walker = &mut resources.walker;
    let forward = queries::forward_of(walker.yaw);
    let right = queries::right_of(walker.yaw);
    let mut wish = forward * asked.y + right * asked.x;
    if wish.norm() > 1.0e-4 {
        wish = wish.normalize() * data::WALK_SPEED;
    }

    walker.velocity.x = wish.x;
    walker.velocity.z = wish.z;
    walker.velocity.y = (walker.velocity.y - data::GRAVITY * delta_time).max(-data::TERMINAL_FALL);

    let (position, velocity, grounded) =
        queries::slide(walker.position, walker.velocity, &solids, delta_time);
    walker.position = position;
    walker.velocity = velocity;
    walker.grounded = grounded;
    if grounded {
        walker.since_ground = 0.0;
    } else {
        walker.since_ground += delta_time;
    }
    if walker.position.y < data::HUB_FLOOR {
        game::restart_hub(walker);
    }
    walker.near_dials = queries::within_dials(walker);
    if resources.walker.near_dials {
        game::queue_cue(&mut resources.story, Cue::NearDials);
    }
}
