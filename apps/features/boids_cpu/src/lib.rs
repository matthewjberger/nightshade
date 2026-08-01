mod components;
mod data;
mod plugin;
mod resources;
mod systems;
mod theme;

use plugin::BoidsPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<resources::BoidsArguments>(
        nightshade_app::Opening::titled("Boids").without_vsync(),
    )
    .add_plugin(nightshade_camera_controller::CameraControllerPlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(BoidsPlugin)
    .run()
}

nightshade_platform::android_main!(launch);
