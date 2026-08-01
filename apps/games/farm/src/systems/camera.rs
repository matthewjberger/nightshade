use crate::components::Upright;
use crate::data::Ask;
use crate::queries::camera::eye_offset;
use crate::resources::{Diorama, Rig};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_camera::queries::camera_look::camera_look_rotation;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

pub fn turn_the_camera(time: Res<Time>, actions: Res<Actions<Ask>>, mut rig: ResMut<Rig>) {
    if actions.just_pressed(Ask::TurnLeft) {
        rig.yaw_index -= 1;
    }
    if actions.just_pressed(Ask::TurnRight) {
        rig.yaw_index += 1;
    }
    let wanted = rig.yaw_index as f32 * std::f32::consts::FRAC_PI_2;
    let share = (theme::CAMERA_TURN * time.delta_time).clamp(0.0, 1.0);
    rig.yaw += (wanted - rig.yaw) * share;
    if (wanted - rig.yaw).abs() < 0.0005 {
        rig.yaw = wanted;
    }
}

pub fn follow_the_farmer(
    time: Res<Time>,
    diorama: Res<Diorama>,
    mut rig: ResMut<Rig>,
    mut cameras: Query<(&mut LocalTransform,)>,
) {
    let Some(camera) = diorama.camera else {
        return;
    };
    let share = (theme::CAMERA_FOLLOW * time.delta_time).clamp(0.0, 1.0);
    let target = diorama.feet;
    let looked_at = rig.focus + (target - rig.focus) * share;
    let yaw = rig.yaw;
    rig.focus = looked_at;
    let eye = looked_at + eye_offset(yaw);
    cameras.for_entity(camera, |(transform,)| {
        transform.translation = eye;
        transform.rotation = camera_look_rotation(eye, looked_at, 0.0);
    });
}

pub fn turn_the_uprights(rig: Res<Rig>, uprights: Query<(&mut Upright, &mut LocalTransform)>) {
    let yaw = rig.yaw;
    let rotation = nalgebra_glm::quat_angle_axis(yaw, &Vec3::y());
    uprights.for_each(|_entity, (upright, transform)| {
        if upright.yaw == yaw {
            return;
        }
        upright.yaw = yaw;
        transform.rotation = rotation;
    });
}
