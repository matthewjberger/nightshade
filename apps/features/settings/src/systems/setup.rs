use crate::data::{CAMERA_DISTANCE, CAMERA_PITCH, CAMERA_TARGET, CAMERA_YAW, Settings};
use crate::resources::SettingsResources;
use crate::systems::panel;
use nightshade::prelude::*;
use nightshade_camera::prelude::*;
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_config::contract::Config;
use nightshade_graphics::prelude::*;
use nightshade_ui_config::prelude::ConfigPanel;

pub fn initialize(
    config: Res<Config<Settings>>,
    mut resources: ResMut<SettingsResources>,
    mut rows: ResMut<ConfigPanel<Settings>>,
    mut render_settings: ResMut<RenderSettings>,
    mut active_camera: ResMut<ActiveCamera>,
    mut commands: Commands,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    render_settings.atmosphere = Atmosphere::Sky;

    spawn_sun(&mut commands);

    let camera = spawn_pan_orbit_camera_at(
        &mut commands,
        CAMERA_TARGET,
        CAMERA_DISTANCE,
        CAMERA_YAW,
        CAMERA_PITCH,
        "Settings Camera".to_string(),
    );
    active_camera.0 = Some(camera);

    let (title_label, built) = panel::build(build.context(), &config);
    resources.title_label = title_label;
    resources.shown_title = config.text(Settings::Title).to_string();
    *rows = built;
}
