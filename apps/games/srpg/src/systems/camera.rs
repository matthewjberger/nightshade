use crate::commands::{tilt_rig, turn_rig, zoom_rig};
use crate::data::Ask;
use crate::resources::{Battlefield, CameraRig, Screen};
use crate::tags::Upright;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_graphics::prelude::RenderSettings;
use nightshade_input::Actions;
use nightshade_platform::contract::input::MouseState;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_state::prelude::*;

pub fn drive_the_rig(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    input: Res<Input>,
    battlefield: Res<Battlefield>,
    mut camera_rig: ResMut<CameraRig>,
    mut render_settings: ResMut<RenderSettings>,
    state_now: Res<State<Screen>>,
    storage: &mut nightshade::ecs::Storage,
) {
    let screen = state_now.current;
    let delta_time = time.delta_time;
    let wheel = if input.mouse.state.contains(MouseState::SCROLLED) {
        input.mouse.wheel_delta.y
    } else {
        0.0
    };
    let turn_left = actions.just_pressed(Ask::TurnLeft);
    let turn_right = actions.just_pressed(Ask::TurnRight);
    let zoom_in = actions.just_pressed(Ask::ZoomIn);
    let zoom_out = actions.just_pressed(Ask::ZoomOut);
    let tilt = actions.just_pressed(Ask::Tilt);
    let camera = battlefield.camera;
    let rig = &mut *camera_rig;

    if matches!(screen, Screen::Title | Screen::Prologue) {
        rig.yaw += theme::TITLE_SPIN * delta_time;
        rig.yaw_index = (rig.yaw / std::f32::consts::FRAC_PI_2).round() as i32;
    } else {
        if turn_left {
            turn_rig(rig, -1);
        }
        if turn_right {
            turn_rig(rig, 1);
        }
        if zoom_in || wheel > 0.0 {
            zoom_rig(rig, -1);
        }
        if zoom_out || wheel < 0.0 {
            zoom_rig(rig, 1);
        }
        if tilt {
            tilt_rig(rig);
        }
        let wanted_yaw = rig.yaw_index as f32 * std::f32::consts::FRAC_PI_2;
        let turn_share = (theme::CAMERA_TURN * delta_time).clamp(0.0, 1.0);
        rig.yaw += (wanted_yaw - rig.yaw) * turn_share;
        if (wanted_yaw - rig.yaw).abs() < 0.0005 {
            rig.yaw = wanted_yaw;
        }
    }

    let zoom_share = (theme::CAMERA_ZOOM * delta_time).clamp(0.0, 1.0);
    let wanted_distance = rig
        .frame
        .unwrap_or(theme::CAMERA_DISTANCES[rig.distance_index]);
    rig.distance += (wanted_distance - rig.distance) * zoom_share;
    let wanted_pitch = theme::CAMERA_PITCHES[rig.pitch_index].to_radians();
    rig.pitch += (wanted_pitch - rig.pitch) * zoom_share;

    let follow_share = (theme::CAMERA_FOLLOW * delta_time).clamp(0.0, 1.0);
    let target = rig.target;
    rig.focus += (target - rig.focus) * follow_share;

    let offset = Vec3::new(
        rig.yaw.sin() * rig.pitch.cos(),
        rig.pitch.sin(),
        rig.yaw.cos() * rig.pitch.cos(),
    ) * rig.distance;
    let eye = rig.focus + offset;
    let looked_at = rig.focus;
    let distance = rig.distance;
    rig.eye = eye;

    if let Some(camera) = camera
        && let Some(transform) = storage.get_mut::<LocalTransform>(camera)
    {
        transform.translation = eye;
        transform.rotation = camera_look_rotation(eye, looked_at, 0.0);
    }
    render_settings.depth_of_field.focus_distance = distance;
}

pub fn turn_the_uprights(
    mut rig: ResMut<CameraRig>,
    uprights: Query<(&mut LocalTransform,), WithTag<Upright>>,
) {
    if (rig.yaw - rig.settled_yaw).abs() < 0.0001 {
        return;
    }
    rig.settled_yaw = rig.yaw;
    let rotation = nalgebra_glm::quat_angle_axis(rig.yaw, &Vec3::y());
    uprights.for_each(|_entity, (transform,)| {
        transform.rotation = rotation;
    });
}
