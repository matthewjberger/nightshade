use crate::data::Ask;
use crate::resources::CoinDashResources;
use crate::systems;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;

pub struct CoinDashPlugin;

impl Plugin for CoinDashPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Coin Dash".to_string();
        app.insert_resource(CoinDashResources::default());
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_system(Stage::Startup, systems::setup::build_overlays);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::menus::update,
                systems::look::update,
                systems::coins::animate,
                systems::play::update,
            ),
        );
    }
}
