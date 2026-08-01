use crate::components::{Anchor, Autopilot, Landed, Target, Velocity};
use crate::contract::{OrreryCommand, OrreryCommands, OrreryEvent};
use crate::data;
use crate::data::Ask;
use crate::queries;
use crate::resources::{Mode, OrreryResources, Screen, View};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;

const MAXIMUM_STEP: f32 = 0.05;
const AUTOPILOT_GAIN: f32 = 1.6;
const AUTOPILOT_TURN: f32 = 3.0;
const AUTOPILOT_CRUISE: f32 = 90.0;
const AUTOPILOT_ARRIVAL: f32 = 2.5;
const AUTOPILOT_CALM: f32 = 1.2;
const STAND_OFF_SCALE: f32 = 3.0;
const STAND_OFF_MARGIN: f32 = 8.0;
const KNOCK_REBOUND: f32 = 0.35;

pub fn fly(
    time: Res<Time>,
    input: Res<Input>,
    actions: Res<Actions<Ask>>,
    window: Res<Window>,
    mut resources: ResMut<OrreryResources>,
    mut orrery_commands: ResMut<OrreryCommands>,
    mut events: EventWriter<OrreryEvent>,
    mut ships: Query<(
        &mut Velocity,
        &mut Landed,
        &mut Anchor,
        &mut Target,
        &mut Autopilot,
        &mut LocalTransform,
    )>,
) {
    let was_landed = resources.pose.landed;
    let elapsed = resources.flight.elapsed;
    let delta_time = time.delta_time.min(MAXIMUM_STEP);
    let flying = resources.flight.mode == Mode::Flying && resources.screen == Screen::Playing;
    let steering = flying && window.cursor_locked;
    let look = input.mouse.raw_mouse_delta;
    let mut pose = resources.pose;
    let mut leave = false;
    let mut match_speed = false;
    let mut view_toggled = false;
    let mut knocks = 0;

    ships.for_each_mut(
        |_entity, (velocity, landed, anchor, target, autopilot, transform)| {
            if flying {
                if actions.just_pressed(Ask::NextTarget) {
                    target.0 = next_target(target.0);
                }
                if actions.just_pressed(Ask::Autopilot) {
                    autopilot.0 = !autopilot.0;
                }
                if actions.just_pressed(Ask::MatchSpeed) {
                    match_speed = true;
                }
                if actions.just_pressed(Ask::SwitchView) {
                    view_toggled = true;
                }
                if actions.just_pressed(Ask::Disembark) && landed.0.is_some() {
                    leave = true;
                }
            }

            let manual = if flying {
                manual_thrust(&actions)
            } else {
                Vec3::zeros()
            };

            if steering {
                let yaw = nalgebra_glm::quat_angle_axis(-look.x * data::SHIP_LOOK_RATE, &Vec3::y());
                let pitch =
                    nalgebra_glm::quat_angle_axis(-look.y * data::SHIP_LOOK_RATE, &Vec3::x());
                let roll =
                    nalgebra_glm::quat_angle_axis(roll_input(&actions) * delta_time, &Vec3::z());
                transform.rotation = (transform.rotation * yaw * pitch * roll).normalize();
            }

            if landed.0.is_some() && manual.magnitude() > 0.0 {
                landed.0 = None;
            }

            if let Some(body) = landed.0 {
                transform.translation = queries::body_position(body, elapsed) + anchor.0;
                velocity.0 = queries::body_velocity(body, elapsed);
                pose.thrust = Vec3::zeros();
            } else {
                let gravity = queries::gravity_at(transform.translation, elapsed);
                let thrust = if autopilot.0 {
                    autopilot_thrust(
                        target.0, velocity.0, autopilot, transform, gravity, elapsed, delta_time,
                    )
                } else {
                    nalgebra_glm::quat_rotate_vec3(&transform.rotation, &manual)
                };
                let drag = queries::drag_at(transform.translation, elapsed);
                pose.thrust = thrust;
                velocity.0 += (gravity + thrust - velocity.0 * drag) * delta_time;
                transform.translation += velocity.0 * delta_time;
                knocks += settle(
                    velocity, landed, anchor, autopilot, transform, thrust, elapsed,
                );
            }

            let contact = queries::nearest_contact(transform.translation, elapsed);
            let target_center = queries::body_position(target.0, elapsed);
            pose.ship_position = transform.translation;
            pose.ship_rotation = transform.rotation;
            pose.ship_velocity = velocity.0;
            pose.landed = landed.0;
            pose.target = target.0;
            pose.autopilot = autopilot.0;
            pose.nearest = contact.body;
            pose.height = contact.height;
            pose.relative_speed =
                (velocity.0 - queries::body_velocity(contact.body, elapsed)).magnitude();
            pose.target_distance = (target_center - transform.translation).magnitude()
                - queries::body_radius(target.0);
        },
    );

    resources.pose = pose;
    resources.flight.knocks += knocks;
    if view_toggled {
        resources.flight.view = match resources.flight.view {
            View::Chase => View::Cockpit,
            View::Cockpit => View::Chase,
        };
    }
    match (was_landed, pose.landed) {
        (None, Some(body)) => events.send(OrreryEvent::Landed { body }),
        (Some(_), None) => events.send(OrreryEvent::Launched),
        _ => {}
    }
    if leave {
        crate::commands::request(&mut orrery_commands, OrreryCommand::LeaveShip);
    }
    if match_speed {
        crate::commands::request(&mut orrery_commands, OrreryCommand::MatchSpeed);
    }
}

fn next_target(current: usize) -> usize {
    let count = data::BODIES.len();
    for step in 1..=count {
        let candidate = (current + step) % count;
        if data::BODIES[candidate].radius > 0.0 {
            return candidate;
        }
    }
    current
}

fn manual_thrust(actions: &Actions<Ask>) -> Vec3 {
    let wish = actions.axis2(Ask::Thrust);
    Vec3::new(
        wish.x * data::SHIP_SIDE_THRUST,
        actions.axis(Ask::Lift) * data::SHIP_SIDE_THRUST,
        -wish.y * data::SHIP_MAIN_THRUST,
    )
}

fn roll_input(actions: &Actions<Ask>) -> f32 {
    actions.axis(Ask::Roll) * data::SHIP_ROLL_RATE
}

fn autopilot_thrust(
    target: usize,
    velocity: Vec3,
    autopilot: &mut Autopilot,
    transform: &mut LocalTransform,
    gravity: Vec3,
    elapsed: f32,
    delta_time: f32,
) -> Vec3 {
    let center = queries::body_position(target, elapsed);
    let radius = queries::body_radius(target);
    let approach = queries::normalized_or(transform.translation - center, Vec3::y());
    let goal = center + approach * (radius * STAND_OFF_SCALE + STAND_OFF_MARGIN);
    let offset = goal - transform.translation;
    let range = offset.magnitude();
    let cruise = (2.0 * data::SHIP_MAIN_THRUST * 0.4 * range)
        .sqrt()
        .min(AUTOPILOT_CRUISE);
    let desired = queries::normalized_or(offset, Vec3::zeros()) * cruise
        + queries::body_velocity(target, elapsed);
    let error = desired - velocity;

    if range < AUTOPILOT_ARRIVAL && error.magnitude() < AUTOPILOT_CALM {
        autopilot.0 = false;
        return Vec3::zeros();
    }

    let mut command = error * AUTOPILOT_GAIN - gravity;
    if command.magnitude() > data::SHIP_MAIN_THRUST {
        command = queries::normalized_or(command, Vec3::zeros()) * data::SHIP_MAIN_THRUST;
    }

    let up = approach;
    let facing = queries::basis_rotation(queries::normalized_or(command, -up), up);
    transform.rotation = queries::blend_rotation(
        transform.rotation,
        facing,
        (AUTOPILOT_TURN * delta_time).min(1.0),
    );
    command
}

fn settle(
    velocity: &mut Velocity,
    landed: &mut Landed,
    resting: &mut Anchor,
    autopilot: &mut Autopilot,
    transform: &mut LocalTransform,
    thrust: Vec3,
    elapsed: f32,
) -> u32 {
    let contact = queries::nearest_contact(transform.translation, elapsed);
    if contact.height > data::SHIP_RADIUS || !contact.solid {
        return 0;
    }
    let center = queries::body_position(contact.body, elapsed);
    let anchor = contact.resting(center, data::SHIP_RADIUS) - center;
    transform.translation = center + anchor;

    if thrust.dot(&contact.up) > 0.0 {
        let radial = velocity.0.dot(&contact.up);
        if radial < 0.0 {
            velocity.0 -= contact.up * radial;
        }
        return 0;
    }

    let carried = queries::body_velocity(contact.body, elapsed);
    let impact = (velocity.0 - carried).magnitude();
    if impact > data::LANDING_SPEED {
        velocity.0 = carried + contact.up * (impact * KNOCK_REBOUND);
        autopilot.0 = false;
        return 1;
    }

    let forward = queries::normalized_or(
        nalgebra_glm::quat_rotate_vec3(&transform.rotation, &Vec3::new(0.0, 0.0, -1.0)),
        Vec3::z(),
    );
    transform.rotation = queries::basis_rotation(forward, contact.up);
    velocity.0 = carried;
    landed.0 = Some(contact.body);
    resting.0 = anchor;
    autopilot.0 = false;
    0
}
