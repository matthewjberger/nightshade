mod ecs;
mod state;
mod systems;
mod theme;

use nightshade::prelude::{App, DefaultPlugins, GamepadPlugin, PhysicsPlugin};
use state::SandboxPlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin)
        .add_plugin(GamepadPlugin)
        .add_plugin(SandboxPlugin)
        .run()
}
