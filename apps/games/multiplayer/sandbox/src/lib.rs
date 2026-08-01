mod commands;
mod components;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;
mod theme;

use nightshade::prelude::App;
use plugin::{SandboxPlugin, ServerPlugin};

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    if nightshade_cli::queries::arguments::parse::<resources::Arguments>()
        .app
        .serve
    {
        return serve();
    }
    nightshade_app::compose::<resources::Arguments>(nightshade_app::Opening::titled(theme::TITLE))
        .add_plugin(nightshade_animation::AnimationPlugin)
        .add_plugin(nightshade_gltf::GltfPlugin)
        .add_plugin(nightshade_text::TextPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(nightshade_physics::PhysicsPlugin)
        .add_plugin(nightshade_gamepad::GamepadPlugin)
        .add_plugin(SandboxPlugin)
        .run()
}

fn serve() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugin(nightshade_log::LogPlugin {
            title: Some(theme::TITLE.to_string()),
            ..Default::default()
        })
        .add_plugin(nightshade_platform::PlatformPlugin)
        .add_plugin(nightshade_scene::ScenePlugin)
        .add_plugin(nightshade_graphics::GraphicsPlugin)
        .add_plugin(nightshade_cli::CliPlugin::<resources::Arguments>::default())
        .add_plugin(nightshade_physics::PhysicsPlugin)
        .add_plugin(ServerPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
