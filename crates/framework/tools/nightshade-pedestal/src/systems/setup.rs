use crate::resources::{Pedestal, PendingModel};
use nightshade::prelude::*;
use nightshade_camera::resources::ActiveCamera;

pub fn initialize(
    mut commands: Commands,
    pedestal: Res<Pedestal>,
    mut active_camera: ResMut<ActiveCamera>,
    mut command_queues: ResMut<nightshade_graphics::resources::queues::CommandQueues>,
    mut pending_model: ResMut<PendingModel>,
    mut loading: ResMut<nightshade_gltf::Loading>,
) {
    let (back, yaw, pitch) = {
        let pedestal = &*pedestal;
        (pedestal.back, pedestal.yaw, pedestal.pitch)
    };
    let camera = {
        let commands = &mut commands;
        nightshade_camera_controller::commands::spawn_pan_orbit_camera_at(
            commands,
            Vec3::zeros(),
            back,
            yaw,
            pitch,
            "Camera".to_string(),
        )
    };
    active_camera.0 = Some(camera);
    let pedestal = *pedestal;
    crate::commands::load_the_sky(&mut command_queues, &pedestal);
    *pending_model = crate::commands::ask_for_the_model(&mut loading, &pedestal);
}
