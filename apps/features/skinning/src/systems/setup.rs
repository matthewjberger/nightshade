use crate::data::FOX_MODEL;
use crate::resources::Model;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_graphics::prelude::*;

pub fn initialize(
    mut commands: Commands,
    mut debug_draw: ResMut<DebugDraw>,
    mut render_settings: ResMut<RenderSettings>,
    mut active_camera: ResMut<ActiveCamera>,
    mut loading: ResMut<nightshade_gltf::Loading>,
    mut model: ResMut<Model>,
) {
    debug_draw.show_grid = true;
    render_settings.atmosphere = Atmosphere::DayNight;

    let camera = {
        let commands = &mut commands;
        spawn_sun(commands);
        spawn_pan_orbit_camera_at(
            commands,
            nalgebra_glm::vec3(0.0, 1.0, 0.0),
            6.0,
            0.6,
            0.4,
            "Main Camera".to_string(),
        )
    };
    active_camera.0 = Some(camera);

    model.asked = Some(nightshade_load::load(
        &mut *loading,
        nightshade_load::Source::Bytes(FOX_MODEL.to_vec()),
    ));
}
