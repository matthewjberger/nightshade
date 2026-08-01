use crate::contract::TemplateCommands;
use crate::data::Ask;
use crate::resources::{Screen, TemplateResources};
use crate::systems;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::*;
use nightshade_ui_menu::prelude::*;

pub struct TemplatePlugin;

impl Plugin for TemplatePlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Template".to_string();
        app.insert_resource(TemplateResources::default());
        app.insert_resource(TemplateCommands::default());
        app.add_plugin(StatePlugin::new(Screen::Title));
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_plugin(MenuPlugin::<Screen>::new(crate::systems::hud::menu_style()));
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.on_enter(Screen::Title, systems::screens::hide_hud);
        app.on_enter(Screen::Playing, systems::screens::show_hud);
        app.on_exit(Screen::Playing, systems::screens::hide_hud);
        app.add_system(Stage::Update, systems::screens::watch_the_keys);
        app.add_system(Stage::Update, systems::screens::drive_the_menus);
        app.add_system(
            Stage::Update,
            while_in(Screen::Playing, systems::example::tick),
        );
        app.add_system(Stage::Update, systems::hud::show_fps);
        app.add_system(Stage::Update, systems::hud::show_viewport);
        app.add_system(Stage::Update, systems::hud::drive_menu);
        app.add_system(Stage::Update, systems::ping::answer_pings);
    }
}
