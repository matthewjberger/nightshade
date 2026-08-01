use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(crate::resources::Bench::default());
        app.add_plugin(nightshade_camera_controller::CameraControllerPlugin);
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_plugin(nightshade_ui::UiPlugin::new());
        app.add_plugin(nightshade_ui_flash::FlashPlugin);
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::Startup, build_the_flash);
        app.add_system(Stage::First, systems::play::take_the_orders);
        app.add_system(Stage::Update, systems::play::show_the_bench);
    }
}

fn build_the_flash(
    mut handles: nightshade::ecs::system_param::ResMut<nightshade_ui_flash::FlashHandles>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let mut tree = nightshade_ui::prelude::UiTreeBuilder::new(build.context());
    *handles = nightshade_ui_flash::commands::add_flash(&mut tree);
    tree.finish();
}
