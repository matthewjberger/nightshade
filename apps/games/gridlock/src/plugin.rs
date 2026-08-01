use crate::data::ask::Ask;
use crate::data::settings::Settings;
use crate::resources::{
    CellJob, City, Flashes, GridlockResources, Indoors, Knocks, Models, Player, Screen, Streaming,
};
use crate::systems;
use nightshade::prelude::*;
use nightshade_camera_controller::systems::controllers::{
    third_person_gamepad_input, third_person_mouse_input, third_person_update_transform,
};
use nightshade_config::prelude::ConfigPlugin;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_random::prelude::Random;
use nightshade_state::prelude::*;
use nightshade_ui_config::prelude::ConfigUiPlugin;
use nightshade_ui_menu::prelude::MenuPlugin;
use nightshade_worker::prelude::{workers_free, workers_hire};

const SEED: u64 = 0x6E1_60A5;
const CREW_LIMIT: usize = 4;

pub struct GridlockPlugin;

impl Plugin for GridlockPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Gridlock".to_string();
        app.insert_resource(GridlockResources {
            seed: SEED,
            ..Default::default()
        });
        app.insert_resource(Player::default());
        app.insert_resource(Models::default());
        app.insert_resource(City::default());
        app.insert_resource(Streaming::default());
        app.insert_resource(Knocks::default());
        app.insert_resource(Indoors::default());
        app.insert_resource(Flashes::default());
        app.insert_resource(Random::seeded(SEED));
        app.insert_resource(workers_hire::<CellJob>(workers_free().clamp(1, CREW_LIMIT)));
        app.add_plugin(InputPlugin::<Ask>::new().persist("gridlock", "bindings.toml"));
        app.add_plugin(ConfigPlugin::<Settings>::new().persist("gridlock", "settings.toml"));
        app.add_plugin(ConfigUiPlugin::<Settings>::new());
        app.add_plugin(StatePlugin::new(Screen::Title));
        app.add_plugin(MenuPlugin::<Screen>::new(systems::interface::menu_style()));

        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::interface::build);
        app.on_enter(Screen::Playing, systems::screens::show_hud);
        app.on_exit(Screen::Playing, systems::screens::hide_hud);
        app.on_enter(Screen::Settings, systems::screens::show_settings);
        app.on_exit(Screen::Settings, systems::screens::hide_settings);

        app.add_system(
            Stage::Update,
            (
                systems::screens::watch_the_keys,
                systems::screens::drive_the_menus,
                systems::tuning::apply,
                systems::models::demand,
                systems::models::collect,
                systems::models::dress,
                systems::stream::feed,
                systems::stream::settle,
                systems::stream::unload,
                systems::lights::dim,
                systems::weather::rain,
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (
                    third_person_mouse_input,
                    third_person_gamepad_input,
                    third_person_update_transform,
                    systems::camera::keep_out_of_the_lots,
                    systems::camera::first_person,
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (
                    systems::fly::toggle,
                    systems::fly::run,
                    systems::walk::run,
                    systems::drive::update,
                    systems::props::knock,
                    systems::props::topple,
                    systems::interiors::enter,
                    systems::board::update,
                    systems::camera::follow,
                    systems::interface::refresh,
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::bounds::measure,
                systems::traffic::populate,
                systems::traffic::flow,
                systems::people::populate,
                systems::people::wander,
                systems::animation::pose,
                systems::flashes::show,
                systems::flashes::rise,
            ),
        );
    }
}
