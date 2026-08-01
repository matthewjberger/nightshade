use crate::data::Ask;
use crate::resources::CoinRushResources;
use crate::systems;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;

pub struct CoinRushPlugin;

impl Plugin for CoinRushPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(nightshade_topdown::TopDownPlugin {
            settings: nightshade_topdown::TopDownSettings {
                speed: crate::theme::PLAYER_SPEED,
            },
        });
        app.add_plugin(nightshade_ui_banner::BannerPlugin::default());
        app.add_plugin(nightshade_pen::PenPlugin);
        app.add_plugin(nightshade_pickup::PickupPlugin);
        app.add_plugin(nightshade_countdown::CountdownPlugin);
        app.add_plugin(nightshade_hover::HoverPlugin {
            settings: nightshade_hover::HoverSettings {
                rate: crate::theme::COIN_RATE,
            },
        });
        app.world.res_mut::<Window>().title = "Coin Rush".to_string();
        app.insert_resource(CoinRushResources::default());
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_system(Stage::Startup, systems::setup::build_overlays);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (systems::menus::update, systems::play::update),
        );
    }
}
