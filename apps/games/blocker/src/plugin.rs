use crate::systems;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_state::prelude::*;
use nightshade_wgpu::RenderAppExt;

pub struct BlockerPlugin;

impl Plugin for BlockerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::resources::Showroom::default());
        app.insert_resource(nightshade_random::Random::seeded(
            blocker_screen::theme::RANDOM_SEED,
        ));
        app.add_system(
            Stage::Startup,
            (systems::setup::raise_the_blocks, systems::setup::initialize),
        );
        app.add_plugin(StatePlugin::new(Screen::Setup));
        app.add_plugin(nightshade_health::HealthPlugin);
        app.add_plugin(blocker_blocks::plugin::BlocksPlugin);
        app.add_plugin(blocker_pocket::plugin::PocketPlugin);
        app.add_plugin(blocker_weather::plugin::WeatherPlugin);
        app.add_plugin(blocker_world::plugin::WorldPlugin);
        app.add_plugin(blocker_farm::plugin::FarmPlugin);
        app.add_plugin(blocker_save::plugin::SavePlugin);
        app.add_plugin(blocker_player::plugin::PlayerPlugin);
        app.add_plugin(blocker_quests::plugin::QuestsPlugin);
        app.add_plugin(blocker_crew::plugin::CrewPlugin);
        app.add_plugin(blocker_chat::plugin::ChatPlugin);
        app.add_plugin(blocker_beasts::plugin::BeastsPlugin);
        app.add_plugin(blocker_barrow::plugin::BarrowPlugin);
        app.add_plugin(blocker_talk::plugin::TalkPlugin);

        app.add_system(Stage::First, systems::world::start_the_world);
        app.add_system(
            Stage::Update,
            (
                systems::menus::handle_the_keys,
                systems::showroom::stand_the_showroom,
                systems::world::look_over_the_world,
            ),
        );
        app.add_system(Stage::LateUpdate, systems::world::raise_the_new_world);
        app.add_plugin(blocker_hud::plugin::HudPlugin);
        app.add_plugin(blocker_screens::plugin::ScreensPlugin);
        order_the_start(app);
        app.add_render_graph_config(systems::curtain::install_curtain_pass);
        app.on_pre_render(systems::curtain::draw_the_curtain);
    }
}

fn order_the_start(app: &mut App) {
    let raise_the_blocks = std::any::type_name_of_val(&systems::setup::raise_the_blocks);
    let initialize = std::any::type_name_of_val(&systems::setup::initialize);
    let read_the_save = std::any::type_name_of_val(&blocker_save::systems::save::read_the_save);
    let take_the_save = std::any::type_name_of_val(&blocker_save::systems::save::take_the_save);
    let sow_the_saved_things =
        std::any::type_name_of_val(&blocker_save::systems::save::sow_the_saved_things);
    let place_player = std::any::type_name_of_val(&blocker_player::systems::arrive::place_player);
    let edges = [
        (initialize, place_player),
        (read_the_save, place_player),
        (initialize, take_the_save),
        (raise_the_blocks, sow_the_saved_things),
    ];
    for (earlier, later) in edges {
        for step in [earlier, later] {
            assert!(
                app.startup.contains(step),
                "the start of blocker names {step}, which no plugin registered"
            );
        }
        app.startup.order(earlier, later);
    }
}
