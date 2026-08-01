use crate::components::{Grounded, Offset, StandingOn, Velocity};
use crate::contract::{OrreryCommand, OrreryCommands};
use crate::data;
use crate::data::Ask;
use crate::queries;
use crate::resources::{Mode, OrreryResources};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;

const MAXIMUM_STEP: f32 = 0.05;

pub fn walk(
    time: Res<Time>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    mut resources: ResMut<OrreryResources>,
    mut orrery_commands: ResMut<OrreryCommands>,
    mut walkers: Query<(
        &StandingOn,
        &mut Offset,
        &mut Velocity,
        &mut Grounded,
        &mut LocalTransform,
    )>,
) {
    if resources.flight.mode != Mode::OnFoot {
        return;
    }

    let look = input.mouse.raw_mouse_delta;
    if window.cursor_locked {
        resources.look.yaw -= look.x * data::FOOT_LOOK_RATE;
        resources.look.pitch = (resources.look.pitch - look.y * data::FOOT_LOOK_RATE)
            .clamp(-data::PITCH_LIMIT, data::PITCH_LIMIT);
    }

    let elapsed = resources.flight.elapsed;
    let delta_time = time.delta_time.min(MAXIMUM_STEP);
    let yaw = resources.look.yaw;
    let ship_position = resources.pose.ship_position;
    let mut pose = resources.pose;
    let mut board = false;

    walkers.for_each_mut(
        |_entity, (standing_on, offset, walker_velocity, grounded, transform)| {
            let center = queries::body_position(standing_on.0, elapsed);
            let contact = queries::body_contact(standing_on.0, center + offset.0, elapsed);
            let up = contact.up;
            let (forward, right) = queries::tangent_frame(up, yaw);

            let wish = actions.axis2(Ask::Walk);
            let mut direction = forward * wish.y + right * wish.x;
            if direction.magnitude() > 0.0 {
                direction = queries::normalized_or(direction, Vec3::zeros());
            }

            let distance = offset.0.magnitude().max(1.0);
            let pull = data::BODIES[standing_on.0].gravity / (distance * distance);
            let radial = walker_velocity.0.dot(&up);
            let mut velocity = direction * data::WALK_SPEED + up * (radial - pull * delta_time);

            if grounded.0 && actions.just_pressed(Ask::Jump) {
                velocity += up * data::JUMP_SPEED;
            }

            walker_velocity.0 = velocity;
            offset.0 += walker_velocity.0 * delta_time;

            let standing = queries::body_contact(standing_on.0, center + offset.0, elapsed);
            let clearance = data::WALKER_HEIGHT * 0.5;
            if standing.height <= clearance {
                offset.0 = standing.resting(center, clearance) - center;
                let downward = walker_velocity.0.dot(&standing.up);
                if downward < 0.0 {
                    walker_velocity.0 -= standing.up * downward;
                }
                grounded.0 = true;
            } else {
                grounded.0 = false;
            }

            let settled = queries::body_contact(standing_on.0, center + offset.0, elapsed);
            let (facing, _) = queries::tangent_frame(settled.up, yaw);
            transform.translation = center + offset.0;
            transform.rotation = queries::basis_rotation(facing, settled.up);

            if actions.just_pressed(Ask::Board)
                && (transform.translation - ship_position).magnitude() < data::BOARD_REACH
            {
                board = true;
            }

            pose.walker_position = transform.translation;
            pose.walker_up = settled.up;
        },
    );

    resources.pose = pose;
    if board {
        crate::commands::request(&mut orrery_commands, OrreryCommand::EnterShip);
    }
}
