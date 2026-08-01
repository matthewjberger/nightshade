use crate::theme;
use nightshade::prelude::*;
use nightshade_camera::prelude::ActiveCamera;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;

pub fn initialize(
    mut commands: Commands,
    mut active_camera: ResMut<ActiveCamera>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut pending_model: ResMut<crate::systems::model::PendingModel>,
    mut render_settings: ResMut<RenderSettings>,
) {
    settle_the_look(&mut render_settings);
    let camera = {
        let commands = &mut commands;
        spawn_pan_orbit_camera_at(
            commands,
            Vec3::zeros(),
            theme::CAMERA_BACK,
            theme::CAMERA_YAW,
            theme::CAMERA_PITCH,
            "Camera".to_string(),
        )
    };
    active_camera.0 = Some(camera);
    let asked = crate::systems::model::ask_for_the_model(&mut loading);
    *pending_model = asked;
}

fn settle_the_look(settings: &mut RenderSettings) {
    settings.atmosphere = Atmosphere::None;
    settings.bloom_enabled = true;
    settings.bloom_intensity = 0.4;
    settings.anti_aliasing = AntiAliasing::Taa;
}
