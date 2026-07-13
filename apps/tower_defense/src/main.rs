mod ecs;
mod retained_ui;
mod state;
mod systems;

use nightshade::prelude::{App, DefaultPlugins};
use state::TowerDefensePlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TowerDefensePlugin)
        .run()
}
