mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod systems;

use nightshade::prelude::*;
use nightshade_camera_controller::CameraControllerPlugin;
use plugin::ManyLightsPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    #[cfg(feature = "wgpu")]
    app.add_plugins(nightshade_wgpu::DefaultPlugins::<nightshade_cli::NoArguments>::new());

    #[cfg(feature = "vulkan")]
    app.add_plugins(nightshade_vulkan::DefaultPlugins::<
        nightshade_cli::NoArguments,
    >::new());

    app.add_plugin(CameraControllerPlugin)
        .add_system(
            Stage::FrameUpdate,
            nightshade_platform::systems::escape_key_exit_system,
        )
        .add_plugin(ManyLightsPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
