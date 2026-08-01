mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;
mod theme;

use plugin::ViewerPlugin;
use resources::ViewerArguments;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app =
        nightshade_app::compose::<ViewerArguments>(nightshade_app::Opening::titled("Viewer"));
    app.add_plugin(nightshade_text::TextPlugin)
        .add_plugin(nightshade_animation::AnimationPlugin)
        .add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_gltf::GltfPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(ViewerPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
