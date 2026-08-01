mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;

use nightshade_camera_controller::CameraControllerPlugin;
use plugin::SkinningPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled(
        "Skinning - whole clip and filtered clip",
    ))
    .add_plugin(nightshade_animation::AnimationPlugin)
    .add_plugin(CameraControllerPlugin)
    .add_plugin(nightshade_gltf::GltfPlugin)
    .add_plugin(SkinningPlugin)
    .run()
}

nightshade_platform::android_main!(launch);
