mod ecs;
mod state;
mod systems;
mod theme;

use nightshade::prelude::{App, DefaultPlugins};
use state::SandboxPlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SandboxPlugin)
        .run()
}
