use nightshade_gamepad::*;
mod commands;
mod components;
mod data;
mod plugin;
mod resources;
mod systems;
mod theme;

use nightshade_physics::PhysicsPlugin;
use plugin::SandboxPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled("Sandbox"),
    );
    app.add_plugin(nightshade_animation::AnimationPlugin)
        .add_plugin(nightshade_gltf::GltfPlugin)
        .add_plugin(nightshade_text::TextPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(PhysicsPlugin)
        .add_plugin(GamepadPlugin)
        .add_plugin(SandboxPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
