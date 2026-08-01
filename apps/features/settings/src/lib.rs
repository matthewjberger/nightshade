mod components;
mod data;
mod plugin;
mod resources;
mod systems;

use plugin::SettingsPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled("Settings"),
    );
    app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(SettingsPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
