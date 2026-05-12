mod ecs;
mod retained_ui;
mod state;
mod systems;

use nightshade::prelude::launch;
use state::TowerDefense;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(TowerDefense::default())
}
