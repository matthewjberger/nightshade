use crate::resources::CoinfallResources;
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;

pub struct CoinfallPlugin;

impl Plugin for CoinfallPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(CoinfallResources::default());
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::Ask>::new());
        app.add_plugin(nightshade_ui_banner::BannerPlugin::default());
        app.add_plugin(nightshade_platformer::PlatformerPlugin {
            settings: nightshade_platformer::PlatformerSettings {
                coyote: theme::COYOTE_SECONDS,
                buffer: theme::BUFFER_SECONDS,
            },
        });
        app.add_plugin(nightshade_pickup::PickupPlugin);
        app.world
            .resource_or_insert_default::<nightshade_plane::PlayPlane>();
        app.world.res_mut::<nightshade_plane::PlayPlane>().0 = nightshade_plane::Plane::Side;
        app.add_system(Stage::Startup, systems::setup::build_interface);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(
            Stage::Update,
            (
                systems::level::load,
                systems::screens::apply,
                systems::menus::update,
                systems::run::restart,
                systems::player::control,
                systems::walkers::patrol,
                systems::walkers::stomp,
                systems::pickups::bob,
                systems::pickups::take_the_coins,
                systems::pickups::reach_the_flag,
                systems::run::tick,
                systems::camera::follow,
                systems::backdrop::drift,
                systems::hud::show,
            ),
        );
    }
}
