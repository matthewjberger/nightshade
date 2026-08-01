mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;

use nightshade_camera_controller::CameraControllerPlugin;
use nightshade_particles::ParticlePlugin;
use plugin::FireworksPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled(
        "Fireworks",
    ))
    .add_plugin(nightshade_text::TextPlugin)
    .add_plugin(ParticlePlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(CameraControllerPlugin)
    .add_plugin(FireworksPlugin)
    .run()
}

nightshade_platform::android_main!(launch);
