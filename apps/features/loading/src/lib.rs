mod commands;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;
mod theme;

use plugin::LoadingPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled(theme::TITLE),
    );
    app.add_plugin(nightshade_audio::AudioPlugin)
        .add_plugin(nightshade_gltf::GltfPlugin)
        .add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(LoadingPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
