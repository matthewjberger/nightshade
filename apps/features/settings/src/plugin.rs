use crate::data::Settings;
use crate::resources::SettingsResources;
use crate::systems;
use nightshade::prelude::*;
use nightshade_config::prelude::ConfigPlugin;
use nightshade_platform::prelude::Window;
use nightshade_ui_config::prelude::ConfigUiPlugin;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.world.res_mut::<Window>().title = "Settings".to_string();
        app.add_plugin(ConfigPlugin::<Settings>::new().persist("nightshade", "settings.toml"));
        app.add_plugin(ConfigUiPlugin::<Settings>::new());
        app.insert_resource(SettingsResources::default());
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Update, systems::scene::keep_the_count);
        app.add_system(Stage::Update, systems::scene::arrange_the_cubes);
        app.add_system(Stage::Update, systems::scene::paint_the_cubes);
        app.add_system(Stage::Update, systems::panel::show_the_title);
    }
}
