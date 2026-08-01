use crate::commands as game;
use crate::components::Extent;
use crate::contract::Dimension;
use crate::data;
use crate::queries;
use crate::resources::FourfoldResources;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::LocalTransform;

pub fn run(
    time: Res<Time>,
    mut resources: ResMut<FourfoldResources>,
    mut commands: Commands,
    blocks: Query<(&Dimension, &Extent, &LocalTransform)>,
) {
    let delta_time = time.delta_time.min(0.05);
    let solids = queries::gather_solids(blocks, Dimension::Glass);

    let drive = resources.drive;
    let runner = &mut resources.runner;
    let wish = drive.wish.x;

    let wanted = wish * data::RUN_SPEED;
    if runner.grounded {
        runner.velocity.x = wanted;
    } else {
        let reach = data::RUN_SPEED * data::AIR_CONTROL * delta_time * 6.0;
        runner.velocity.x += (wanted - runner.velocity.x).clamp(-reach, reach);
    }

    let can_jump = runner.grounded || runner.since_ground < data::COYOTE_TIME;
    if drive.jump && can_jump {
        runner.velocity.y = data::RUN_JUMP;
        runner.grounded = false;
        runner.since_ground = data::COYOTE_TIME;
    }
    runner.velocity.y =
        (runner.velocity.y - data::RUN_GRAVITY * delta_time).max(-data::TERMINAL_FALL);
    runner.velocity.z = 0.0;

    let (position, velocity, grounded) =
        queries::slide(runner.position, runner.velocity, &solids, delta_time);
    runner.position = Vec3::new(position.x, position.y, 0.0);
    runner.velocity = velocity;
    runner.grounded = grounded;
    if grounded {
        runner.since_ground = 0.0;
    } else {
        runner.since_ground += delta_time;
    }

    if runner.position.x > data::GLASS_ARMED_AT {
        runner.armed = true;
    }
    if runner.position.y < data::GLASS_FLOOR {
        game::fall_back(runner);
    }

    let catch = (data::GLASS_CATCH * delta_time).clamp(0.0, 1.0);
    runner.focus += (runner.position - runner.focus) * catch;

    let standing = runner.position + Vec3::new(0.0, data::RUNNER_SIZE.y * 0.5, 0.0);
    if resources.avatar != Entity::default() {
        commands.set(
            resources.avatar,
            LocalTransform {
                translation: standing,
                rotation: Quat::identity(),
                scale: data::RUNNER_SIZE,
            },
        );
    }
}
