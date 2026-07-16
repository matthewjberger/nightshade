mod ecs;
mod plugin;
mod systems;

use nightshade::prelude::{App, DefaultPlugins};
use plugin::GalleryPlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(GalleryPlugin)
        .run()
}
