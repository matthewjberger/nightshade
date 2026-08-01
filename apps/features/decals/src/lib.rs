mod plugin;
mod resources;
mod systems;
mod theme;

use plugin::DecalsDemoPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled("Decals Demo"),
    );
    app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_text::TextPlugin)
        .add_plugin(nightshade_decals::DecalsPlugin)
        .add_plugin(nightshade_gltf::GltfPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(DecalsDemoPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
