use crate::resources::Board;
use crate::systems;
use crate::theme;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board {
            shaped: theme::SHAPE_START,
            ..Default::default()
        });
        app.add_plugin(nightshade_readout::ReadoutPlugin::titled(theme::TITLE));
        app.add_plugin(nightshade_ui::UiPlugin::new());
        app.add_plugin(nightshade_ui_nodegraph::NodeGraphPlugin::default());
        app.add_system(Stage::Startup, systems::setup::stand_it_up);
        app.add_system(Stage::Startup, systems::setup::spread_the_board);
        app.add_system(Stage::Update, systems::play::take_the_orders);
        app.add_system(Stage::Update, systems::play::drag_the_field);
        app.add_system(Stage::Update, systems::play::count_the_changes);
        app.add_system(Stage::Update, systems::play::show_the_board);
    }
}
