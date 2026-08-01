mod plugin;
mod systems;
mod theme;

use plugin::DeviceLossPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled(theme::TITLE).settling_at(theme::SETTLED),
    );
    app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_gltf::GltfPlugin)
        .add_plugin(DeviceLossPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
