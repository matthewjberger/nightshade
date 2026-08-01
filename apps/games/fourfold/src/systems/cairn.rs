use crate::commands as game;
use crate::components::Extent;
use crate::contract::{Cue, Dimension};
use crate::data;
use crate::queries;
use crate::resources::FourfoldResources;
use nightshade::prelude::*;
use nightshade_graphics::commands::builtins::set_mesh_material;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

pub fn walk(
    time: Res<Time>,
    mut resources: ResMut<FourfoldResources>,
    blocks: Query<(&Dimension, &Extent, &LocalTransform)>,
) {
    let delta_time = time.delta_time.min(0.05);
    let solids = queries::gather_solids(blocks, Dimension::Cairn);
    let drive = resources.drive;
    let digger = &mut resources.digger;

    digger.yaw -= drive.look.x;
    digger.pitch = (digger.pitch - drive.look.y).clamp(-data::PITCH_LIMIT, data::PITCH_LIMIT);

    let forward = queries::forward_of(digger.yaw);
    let right = queries::right_of(digger.yaw);
    let mut wish = forward * drive.wish.y + right * drive.wish.x;
    if wish.norm() > 1.0e-4 {
        wish = wish.normalize() * data::WALK_SPEED;
    }
    digger.velocity.x = wish.x;
    digger.velocity.z = wish.z;

    let can_jump = digger.grounded || digger.since_ground < data::COYOTE_TIME;
    if drive.jump && can_jump {
        digger.velocity.y = data::JUMP_SPEED;
        digger.grounded = false;
        digger.since_ground = data::COYOTE_TIME;
    }
    digger.velocity.y = (digger.velocity.y - data::GRAVITY * delta_time).max(-data::TERMINAL_FALL);

    let (position, velocity, grounded) =
        queries::slide(digger.position, digger.velocity, &solids, delta_time);
    digger.position = position;
    digger.velocity = velocity;
    digger.grounded = grounded;
    if grounded {
        digger.since_ground = 0.0;
    } else {
        digger.since_ground += delta_time;
    }
    if digger.position.y < data::HUB_FLOOR {
        digger.position = data::CAIRN_START;
        digger.velocity = Vec3::zeros();
    }
}

pub fn reach(mut resources: ResMut<FourfoldResources>, mut commands: Commands) {
    let index = queries::tone_index(Dimension::Cairn);
    if resources.lit[index] {
        return;
    }
    let flat = Vec3::new(
        resources.digger.position.x - data::CAIRN_PEAK.x,
        0.0,
        resources.digger.position.z - data::CAIRN_PEAK.z,
    );
    if flat.norm() > data::CAIRN_REACH {
        return;
    }
    resources.lit[index] = true;
    game::queue_cue(&mut resources.story, Cue::Solved);
    if resources.frames[index] != Entity::default() {
        set_mesh_material(
            &mut commands,
            resources.frames[index],
            data::TONES[index].material,
        );
    }
}
