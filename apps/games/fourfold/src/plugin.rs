use crate::contract::Dimension;
use crate::data;
use crate::resources::{FourfoldResources, Screen};
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::Window;
use nightshade_state::prelude::*;
use nightshade_ui_menu::prelude::MenuPlugin;

pub struct FourfoldPlugin;

impl Plugin for FourfoldPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = data::TITLE.to_string();
        app.insert_resource(FourfoldResources::default());
        app.add_plugin(StatePlugin::new(Screen::Title));
        app.add_plugin(StatePlugin::new(Dimension::Hub));
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::Ask>::new());
        app.add_plugin(MenuPlugin::<Screen>::new(systems::menu::menu_style()));
        app.add_plugin(nightshade_portal::prelude::PortalPlugin);

        app.add_system(Stage::Startup, systems::autoplay::arm);
        app.add_system(Stage::Startup, systems::setup::build_overlay);
        app.add_system(Stage::Startup, systems::setup::initialize);

        app.on_enter(Screen::Title, systems::menu::release_the_cursor);
        app.on_enter(Screen::Title, systems::hud::hide_hud);
        app.on_enter(Screen::Paused, systems::menu::release_the_cursor);
        app.on_enter(Screen::Playing, systems::menu::hold_the_cursor);
        app.on_enter(Screen::Playing, systems::hud::show_hud);
        app.on_exit(Screen::Playing, systems::hud::hide_hud);

        app.on_enter(Dimension::Hub, systems::cross::enter_hub);
        app.on_enter(Dimension::Glass, systems::cross::enter_glass);
        app.on_enter(Dimension::Loom, systems::cross::enter_loom);
        app.on_enter(Dimension::Cairn, systems::cross::enter_cairn);
        app.on_enter(Dimension::Hearth, systems::cross::enter_hearth);

        app.add_system(Stage::Update, systems::menu::update);
        app.add_system(Stage::Update, systems::autoplay::toggle);
        app.add_system(Stage::Update, systems::autoplay::open_the_door);
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::menu::regrab),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                (systems::drive::from_hands, systems::autoplay::play),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                while_in(
                    Dimension::Hub,
                    (
                        systems::hub::look,
                        systems::hub::walk,
                        systems::arch::turn,
                        systems::arch::open,
                        systems::arch::descend,
                        systems::cross::leave_hub,
                        systems::view::place_hub_camera,
                    ),
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                while_in(
                    Dimension::Glass,
                    (
                        systems::glass::run,
                        systems::cross::leave_glass,
                        systems::view::place_glass_camera,
                    ),
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                while_in(
                    Dimension::Loom,
                    (
                        systems::loom::step,
                        systems::loom::settle,
                        systems::loom::carry,
                        systems::cross::leave_loom,
                    ),
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                while_in(
                    Dimension::Cairn,
                    (
                        systems::cairn::walk,
                        systems::cairn::reach,
                        systems::cross::leave_cairn,
                        systems::view::place_cairn_camera,
                    ),
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(
                Screen::Playing,
                while_in(
                    Dimension::Hearth,
                    (
                        systems::hearth::walk,
                        systems::hearth::draw,
                        systems::hearth::reach,
                        systems::cross::leave_hearth,
                        systems::view::place_hearth_camera,
                    ),
                ),
            ),
        );
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::story::tell),
        );
        app.add_system(Stage::Update, while_in(Screen::Playing, systems::hud::show));
    }
}
