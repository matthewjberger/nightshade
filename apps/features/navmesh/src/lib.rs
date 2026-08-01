mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;

use nightshade_camera_controller::CameraControllerPlugin;
use nightshade_navmesh::NavmeshPlugin;
use nightshade_physics::PhysicsPlugin;
use plugin::NavMeshDemoPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    nightshade_app::compose::<nightshade_app::NoArguments>(nightshade_app::Opening::titled(
        "NavMesh Pathfinding",
    ))
    .add_plugin(nightshade_animation::AnimationPlugin)
    .add_plugin(nightshade_text::TextPlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(PhysicsPlugin)
    .add_plugin(NavmeshPlugin)
    .add_plugin(CameraControllerPlugin)
    .add_plugin(nightshade_gltf::GltfPlugin)
    .add_plugin(NavMeshDemoPlugin)
    .run()
}

nightshade_platform::android_main!(launch);
