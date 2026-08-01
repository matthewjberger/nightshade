use crate::components::{Grounded, Landed, Offset, StandingOn, Velocity};
use crate::contract::{OrreryCommand, OrreryCommands};
use crate::data;
use crate::queries;
use crate::resources::{Mode, OrreryResources};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;

const STEP_ASIDE: f32 = 2.6;

pub fn apply(
    mut resources: ResMut<OrreryResources>,
    mut orrery_commands: ResMut<OrreryCommands>,
    mut commands: Commands,
    mut ships: Query<(&mut Velocity, &Landed)>,
    mut walkers: Query<(&mut StandingOn, &mut Offset, &mut Velocity, &mut Grounded)>,
) {
    let pending = std::mem::take(&mut orrery_commands.pending);
    if pending.is_empty() {
        return;
    }

    let elapsed = resources.flight.elapsed;
    let pose = resources.pose;
    let walker_entity = resources.scene.walker;

    for command in pending {
        match command {
            OrreryCommand::MatchSpeed => {
                let velocity = queries::body_velocity(pose.nearest, elapsed);
                ships.for_each_mut(|_entity, (ship_velocity, landed)| {
                    if landed.0.is_none() {
                        ship_velocity.0 = velocity;
                    }
                });
            }
            OrreryCommand::LeaveShip => {
                let Some(body) = pose.landed else {
                    continue;
                };
                let center = queries::body_position(body, elapsed);
                let aside = nalgebra_glm::quat_rotate_vec3(&pose.ship_rotation, &Vec3::x());
                let spot = pose.ship_position + aside * STEP_ASIDE - center;
                let floor = queries::body_radius(body) + data::WALKER_HEIGHT * 0.5;
                let offset = queries::normalized_or(spot, Vec3::y()) * floor;
                walkers.for_each_mut(
                    |_entity, (standing_on, walker_offset, walker_velocity, grounded)| {
                        standing_on.0 = body;
                        walker_offset.0 = offset;
                        walker_velocity.0 = Vec3::zeros();
                        grounded.0 = true;
                    },
                );
                commands.set(walker_entity, Visibility { visible: true });
                resources.look.pitch = 0.0;
                resources.pose.walker_position = center + offset;
                resources.pose.walker_up = queries::normalized_or(offset, Vec3::y());
                crate::commands::set_mode(&mut resources, Mode::OnFoot);
            }
            OrreryCommand::EnterShip => {
                commands.set(walker_entity, Visibility { visible: false });
                crate::commands::set_mode(&mut resources, Mode::Flying);
            }
        }
    }
}
