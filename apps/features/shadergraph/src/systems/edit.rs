use crate::data::Ask;
use crate::resources::Chosen;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, Res, ResMut};
use nightshade_input::Actions;
use nightshade_ui_nodegraph::prelude::{
    GraphGrip, Holding, NodeGraphSettings, cut_the_node, what_is_under,
};

pub fn cut_what_is_under(
    settings: Res<NodeGraphSettings>,
    grip: Res<GraphGrip>,
    asks: Res<Actions<Ask>>,
    mut chosen: ResMut<Chosen>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    if !asks.just_pressed(Ask::Cut) || !grip.inside {
        return;
    }
    let Holding::Node(node) = what_is_under(storage, &settings, grip.pointer) else {
        return;
    };
    cut_the_node(&mut commands, storage, node);
    if chosen.0 == Some(node) {
        chosen.0 = None;
    }
}
