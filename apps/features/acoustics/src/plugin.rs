use crate::resources::AcousticsDemo;
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct AcousticsDemoPlugin;

impl Plugin for AcousticsDemoPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Ray Traced Audio".to_string();
        app.insert_resource(AcousticsDemo::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Update, systems::view::hold_the_view);
        app.add_system(
            Stage::Update,
            (
                systems::door::swing_the_door,
                systems::echo::show_the_echoes,
                systems::rays::flow_the_rays,
                systems::panel::refresh,
            ),
        );
        #[cfg(not(target_arch = "wasm32"))]
        app.add_system(
            Stage::Update,
            (
                systems::sound::start_the_tone,
                systems::sound::mark_the_sounding,
                systems::sound::steer_the_rain,
            ),
        );
    }
}
