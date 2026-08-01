use nightshade_camera_controller::CameraControllerPlugin;
use nightshade_particles::ParticlePlugin;
mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;

use plugin::TowerDefensePlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled(
        "Tower Defense",
    ))
    .add_plugin(nightshade_text::TextPlugin)
    .add_plugin(ParticlePlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(CameraControllerPlugin)
    .add_plugin(TowerDefensePlugin)
    .run()
}

nightshade_platform::android_main!(launch);
