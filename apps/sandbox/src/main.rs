mod ecs;
mod plugin;
mod systems;
mod theme;

use nightshade::prelude::{App, DefaultPlugins, GamepadPlugin, PhysicsPlugin, TerrainPlugin};
use plugin::SandboxPlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TerrainPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(GamepadPlugin)
        .add_plugin(SandboxPlugin)
        .run()
}
