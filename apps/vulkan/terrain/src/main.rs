mod commands;
mod components;
mod plugin;
mod queries;
mod resources;
mod systems;
mod theme;

use nightshade::prelude::App;
use plugin::TerrainPlugin;
use resources::Arguments;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugin(nightshade_vulkan::RenderPlugin::default())
        .add_plugin(nightshade_window::WindowPlugin::default())
        .add_plugin(nightshade_cli::CliPlugin::<Arguments>::default())
        .add_plugin(nightshade_log::LogPlugin::default())
        .add_plugin(nightshade_camera::CameraPlugin)
        .add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_system(
            nightshade::app::Stage::FrameUpdate,
            nightshade_platform::systems::escape_key_exit_system,
        )
        .add_plugin(TerrainPlugin)
        .run()
}
