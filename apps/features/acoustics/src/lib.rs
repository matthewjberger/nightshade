mod commands;
mod data;
mod plugin;
mod resources;
mod systems;

use plugin::AcousticsDemoPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled("Ray Traced Audio"),
    );
    app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_audio::AudioPlugin)
        .add_plugin(nightshade_acoustics::AcousticsPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(AcousticsDemoPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
