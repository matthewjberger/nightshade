mod ecs;
mod plugin;
mod retained_ui;
mod systems;

use nightshade::prelude::{App, CameraControllerPlugin, DefaultPlugins, ExitOnEscapePlugin};
use plugin::TowerDefensePlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ExitOnEscapePlugin)
        .add_plugin(CameraControllerPlugin)
        .add_plugin(TowerDefensePlugin)
        .run()
}
