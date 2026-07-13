mod ecs;
mod state;
mod systems;

use nightshade::prelude::{App, DefaultPlugins};
use state::GalleryPlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(GalleryPlugin)
        .run()
}
