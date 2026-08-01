mod data;
mod plugin;
mod resources;
mod systems;
mod theme;

use plugin::LiquidsPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled(theme::TITLE).settling_at(theme::SETTLED),
    )
    .add_plugin(nightshade_camera_controller::CameraControllerPlugin)
    .add_plugin(LiquidsPlugin)
    .run()
}

nightshade_platform::android_main!(launch);
