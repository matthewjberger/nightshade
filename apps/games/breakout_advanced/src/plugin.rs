use crate::data;
use crate::resources::{BreakoutAdvancedResources, Screen};
use crate::systems;
use nightshade::prelude::*;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui_menu::prelude::*;

pub struct BreakoutAdvancedPlugin;

impl Plugin for BreakoutAdvancedPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = data::TITLE.to_string();
        app.insert_resource(BreakoutAdvancedResources::default());
        app.add_plugin(StatePlugin::new(Screen::Title));
        app.add_plugin(nightshade_input::InputPlugin::<crate::data::Ask>::new());
        app.add_plugin(MenuPlugin::<Screen>::new(systems::screens::menu_style()));
        app.add_system(Stage::Startup, systems::hud::build_overlay);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Startup, systems::setup::upload_debris_sprite);
        app.on_enter(Screen::Playing, systems::screens::show_hud);
        app.on_exit(Screen::Playing, systems::screens::hide_hud);
        app.add_system(
            Stage::Update,
            (
                systems::screens::watch_the_keys,
                systems::screens::drive_the_menus,
                systems::pad::navigate,
                systems::juice::settle,
                systems::round::rebuild,
                while_in(Screen::Playing, systems::round::tick),
                while_in(Screen::Playing, systems::paddle::drive),
                while_in(Screen::Playing, systems::ball::advance),
                systems::juice::react,
                systems::round::watch,
                systems::ball::shape,
                while_in(Screen::Playing, systems::trail::shed),
                systems::trail::fade,
                systems::bricks::animate,
                systems::camera::frame,
                systems::hud::show,
            ),
        );
    }
}
