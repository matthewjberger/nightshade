mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;
mod theme;

use plugin::WorkersPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled(
        theme::TITLE,
    ))
    .add_plugin(nightshade_camera_controller::CameraControllerPlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(WorkersPlugin)
    .run()
}

nightshade_platform::android_main!(launch);
