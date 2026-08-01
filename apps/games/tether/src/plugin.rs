use crate::resources::{Screen, TetherResources};
use crate::systems;
use nightshade::prelude::*;
use nightshade_camera_controller::systems::controllers::first_person_camera_look_system;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui_menu::prelude::MenuPlugin;

pub struct TetherPlugin;

impl Plugin for TetherPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Tether".to_string();
        app.insert_resource(TetherResources::default());
        app.add_plugin(StatePlugin::new(Screen::Title));
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::Ask>::new());
        app.add_plugin(MenuPlugin::<Screen>::new(systems::hud::menu_style()));

        app.add_system(Stage::Startup, systems::setup::shape_the_air);
        app.add_system(Stage::Startup, systems::setup::build_the_canyon);
        app.add_system(Stage::Startup, systems::setup::build_the_interface);

        app.on_enter(Screen::Title, systems::screens::hide_hud);
        app.on_enter(Screen::Playing, systems::screens::start_the_world);
        app.on_enter(Screen::Playing, systems::screens::show_hud);
        app.on_exit(Screen::Playing, systems::screens::stop_the_world);
        app.on_exit(Screen::Playing, systems::screens::hide_hud);

        app.add_system(Stage::Update, systems::screens::watch_the_keys);
        app.add_system(
            Stage::Update,
            while_in_any(
                [Screen::Title, Screen::Paused],
                systems::screens::drive_the_menus,
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (
                    first_person_camera_look_system,
                    systems::tether::aim_the_line,
                    systems::tether::throw_the_line,
                    systems::motion::drive_the_runner,
                    systems::motion::catch_the_falls,
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            (
                systems::camera::follow_the_runner,
                systems::rope::draw_the_line,
                systems::beacons::bob_the_beacons,
                systems::hud::read_the_releases,
                systems::hud::show_the_speed,
                systems::hud::show_the_reticle,
            ),
        );
    }
}
