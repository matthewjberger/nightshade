mod handles;
mod state;
mod systems;
mod theme;

use nightshade::prelude::launch;
use state::Sandbox;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(Sandbox::default())
}
