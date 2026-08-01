use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct ManyLightsPlugin;

impl Plugin for ManyLightsPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title =
            "Many Lights - Clustered Forward Rendering".to_string();
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Update, systems::orbit::carry_the_lights);
    }
}
