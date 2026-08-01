use crate::data::{Ask, Settings};
use crate::resources::{Arguments, Bulwark, Opening, Screen};
use crate::systems;
use crate::theme;
use nightshade::prelude::*;
use nightshade_config::prelude::ConfigPlugin;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui_config::prelude::ConfigUiPlugin;
use nightshade_ui_menu::prelude::MenuPlugin;

pub struct BulwarkPlugin;

impl Plugin for BulwarkPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = theme::TITLE.to_string();
        app.insert_resource(Bulwark::default());
        app.add_plugin(ConfigPlugin::<Settings>::new().persist("nightshade", "bulwark.toml"));
        app.add_plugin(ConfigUiPlugin::<Settings>::new());
        let opening = match app.world.res::<Arguments>().open {
            Opening::Title => Screen::Title,
            Opening::Settings => Screen::Options,
            Opening::Board => Screen::Playing,
        };
        app.add_plugin(StatePlugin::new(opening));
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_plugin(MenuPlugin::<Screen>::new(systems::setup::menu_style()));

        app.add_plugin(nightshade_waves::WavesPlugin);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::setup::lay_the_plan);

        app.on_enter(Screen::Title, systems::screens::hide_the_hud);
        app.on_enter(Screen::Title, systems::board::rest_the_board);
        app.on_enter(Screen::Title, systems::board::sweep_the_crystals);
        app.on_enter(Screen::Options, systems::screens::hide_the_hud);
        app.on_enter(Screen::Playing, systems::screens::show_the_hud);
        app.on_enter(Screen::Playing, systems::board::march_the_board);
        app.on_enter(Screen::Paused, systems::board::rest_the_board);
        app.on_enter(Screen::Defeat, systems::screens::hide_the_hud);
        app.on_enter(Screen::Defeat, systems::screens::tell_the_ending);
        app.on_enter(Screen::Defeat, systems::board::sweep_the_crystals);
        app.on_enter(Screen::Defeat, systems::board::rest_the_board);
        app.on_enter(Screen::Victory, systems::screens::hide_the_hud);
        app.on_enter(Screen::Victory, systems::screens::tell_the_ending);
        app.on_enter(Screen::Victory, systems::board::sweep_the_crystals);
        app.on_enter(Screen::Victory, systems::board::rest_the_board);
        app.on_exit(Screen::Title, systems::board::sweep_the_crystals);
        app.on_exit(Screen::Title, systems::board::clear_the_board);
        app.on_exit(Screen::Title, systems::board::pull_the_towers);
        app.on_exit(Screen::Defeat, systems::board::clear_the_board);
        app.on_exit(Screen::Defeat, systems::board::pull_the_towers);
        app.on_exit(Screen::Victory, systems::board::clear_the_board);
        app.on_exit(Screen::Victory, systems::board::pull_the_towers);

        app.add_system(Stage::Update, systems::screens::watch_the_keys);
        app.add_system(Stage::Update, systems::screens::drive_the_menus);
        app.add_system(Stage::Update, systems::screens::name_the_board);
        app.add_system(Stage::Update, systems::screens::show_the_best);
        app.add_system(Stage::Update, systems::screens::dim_the_board);
        app.add_system(Stage::Update, systems::board::lay_the_chosen);
        app.add_system(Stage::Update, systems::board::pulse_the_gate);
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::board::mend_the_keep),
        );
        app.add_system(Stage::Update, systems::board::hold_the_zoom);
        app.add_system(Stage::Update, systems::board::rock_the_camera);
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::build::pick_the_kind),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::build::aim_the_marker),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::build::raise_the_tower),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::build::work_the_tower),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::build::show_the_ring),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::build::show_the_ghost),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::waves::set_the_pace),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::waves::hurry_the_march),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::waves::meet_the_arrivals),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::waves::count_the_cleared),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::crystals::walk_the_road),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::towers::pick_and_fire),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::beads::fly_and_strike),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::towers::settle_the_stone),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::crystals::cool_the_flash),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::crystals::chill_the_crystals),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::crystals::dress_the_crystals),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::hud::show_the_chips),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::hud::show_the_hotbar),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::hud::show_the_banner),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::hud::show_the_inspection),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::hud::show_the_send),
        );
    }
}
