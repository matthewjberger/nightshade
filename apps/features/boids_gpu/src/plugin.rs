use crate::resources::BoidsGpuResources;
use crate::systems::{hud, input, lifecycle};
use nightshade::prelude::*;

pub struct BoidsGpuPlugin;

impl Plugin for BoidsGpuPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoidsGpuResources::default());
        app.add_system(Stage::Startup, lifecycle::initialize);
        app.add_system(Stage::Update, (input::handle_events, hud::update));
    }
}
