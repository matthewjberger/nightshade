use crate::queries::{aim_points_down, direction_to_world, hand_pose, palm_axis, palm_direction};
use crate::resources::Jarvis;
use crate::theme::{DRAG, MAX_SPEED, SNAP_TURN_RADIANS, THRUST};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_openxr::contract::{XrHand, XrInput, XrRig};
use nightshade_platform::prelude::*;

const FLARE_BASE: f32 = 0.02;
const FLARE_REACH: f32 = 0.55;
const FLARE_WIDTH: f32 = 0.05;
const FLOOR: f32 = 0.0;
const TURN_ARM: f32 = 0.6;
const TURN_RELEASE: f32 = 0.3;

pub fn fly_suit(
    mut jarvis: ResMut<Jarvis>,
    xr_input: Res<XrInput>,
    rig: Option<Res<XrRig>>,
    time: Res<Time>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(rig) = rig.map(|rig| *rig) else {
        return;
    };
    let delta_time = time.delta_time.min(0.05);
    let origin = crate::queries::origin_pose(storage, &rig);

    let mut thrust_stage = Vec3::zeros();
    for hand in XrHand::ALL {
        let state = XrRig::hand_state(&xr_input, hand);
        let throttle = if state.tracked && !aim_points_down(&state) {
            state.trigger
        } else {
            0.0
        };
        if let Some(pose) = hand_pose(storage, &rig, hand) {
            thrust_stage -= palm_direction(hand, &pose.orientation) * throttle * THRUST;
        }
        let flare = jarvis.gauntlet_mut(hand).flare;
        if let Some(transform) = storage.get_mut::<LocalTransform>(flare) {
            let length = FLARE_BASE + FLARE_REACH * throttle;
            let width = FLARE_BASE + FLARE_WIDTH * throttle;
            transform.scale = Vec3::new(width, length, width);
            transform.translation = palm_axis(hand) * (0.05 + length * 0.5);
        }
    }

    let thrust_world = direction_to_world(&origin, thrust_stage);
    let previous_velocity = jarvis.velocity;
    let mut velocity = previous_velocity + thrust_world * delta_time;
    velocity *= (1.0 - DRAG * delta_time).max(0.0);
    let speed = velocity.norm();
    if speed > MAX_SPEED {
        velocity *= MAX_SPEED / speed;
    }
    jarvis.acceleration = if delta_time > 0.0 {
        (velocity - previous_velocity).norm() / delta_time
    } else {
        0.0
    };

    let turn_axis = xr_input.right.thumbstick.x;
    let mut turn = 0.0;
    if turn_axis.abs() > TURN_ARM {
        if !jarvis.turn_armed {
            turn = -turn_axis.signum() * SNAP_TURN_RADIANS;
            jarvis.turn_armed = true;
        }
    } else if turn_axis.abs() < TURN_RELEASE {
        jarvis.turn_armed = false;
    }

    if let Some(transform) = storage.get_mut::<LocalTransform>(rig.origin) {
        transform.translation += velocity * delta_time;
        if transform.translation.y < FLOOR {
            transform.translation.y = FLOOR;
            if velocity.y < 0.0 {
                velocity.y = 0.0;
            }
        }
        if turn != 0.0 {
            transform.rotation =
                nalgebra_glm::quat_angle_axis(turn, &Vec3::y()) * transform.rotation;
        }
    }
    jarvis.velocity = velocity;
}
