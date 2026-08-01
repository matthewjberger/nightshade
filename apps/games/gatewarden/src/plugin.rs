use crate::data::ask::Ask;
use crate::resources::GatewardenResources;
use crate::systems;
use nightshade::prelude::*;
use nightshade_input::InputPlugin;
use nightshade_wgpu::RenderAppExt;

pub struct GatewardenPlugin;

impl Plugin for GatewardenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GatewardenResources::default());
        app.add_plugin(InputPlugin::<Ask>::new());
        app.add_render_graph_config(systems::backdrop::install_forest_pass);
        app.on_pre_render(systems::backdrop::advance_forest);
        app.add_system(Stage::Startup, systems::setup::build_overlays);
        app.add_system(Stage::Startup, systems::setup::initialize);
        app.add_system(Stage::Update, systems::menus::update);
        app.add_system(Stage::Update, systems::tutorial::update);
        app.add_system(Stage::Update, systems::battle::update);
        app.add_system(Stage::Update, systems::theater::update);
        app.add_system(Stage::Update, systems::table::sync);
        app.add_system(Stage::Update, systems::arrow::sync);
        app.add_system(Stage::Update, systems::gauges::sync);
        app.add_system(Stage::Update, systems::bazaar::sync);
        app.add_system(Stage::Update, systems::hud::sync);
        app.add_system(Stage::Update, systems::effects::update);
        app.add_system(Stage::Update, systems::viewport::scale);
    }
}
