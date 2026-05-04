mod ecs;
mod state;
mod systems;

use nightshade::prelude::launch;
use state::Gallery;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Gallery::default())
}
