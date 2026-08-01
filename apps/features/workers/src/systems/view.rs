use crate::theme;
use nightshade::prelude::*;
use nightshade_camera_controller::contract::PanOrbitCamera;
use nightshade_platform::prelude::Time;

pub fn turn_the_camera(time: Res<Time>, mut cameras: Query<(&mut PanOrbitCamera,)>) {
    let step = theme::CAMERA_TURN * time.delta_time;
    cameras.for_each_mut(|_entity, (camera,)| {
        camera.yaw += step;
        camera.target_yaw = camera.yaw;
    });
}
