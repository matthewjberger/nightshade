use crate::resources::{GraphGrip, GraphView, NodeGraphSettings};
use crate::systems::grip::take_the_grip;
use crate::systems::hold::move_what_is_held;
use crate::systems::paint::paint_the_graph;
use crate::systems::settle::settle_the_wires;
use nightshade::app::{App, Stage};
use nightshade::plugin::Plugin;

#[derive(Default)]
pub struct NodeGraphPlugin {
    pub settings: NodeGraphSettings,
}

impl Plugin for NodeGraphPlugin {
    fn needs(&self) -> Vec<nightshade::plugin::Need> {
        nightshade::needs![
            nightshade_platform::contract::input::Input => "PlatformPlugin",
        ]
    }

    fn build(&self, app: &mut App) {
        app.insert_resource(self.settings.clone());
        app.insert_resource(GraphView::default());
        app.insert_resource(GraphGrip::default());
        app.add_system(Stage::First, take_the_grip);
        app.add_system(Stage::Update, move_what_is_held);
        app.add_system(Stage::LateUpdate, settle_the_wires);
        app.add_system(Stage::LateUpdate, paint_the_graph);
    }
}
