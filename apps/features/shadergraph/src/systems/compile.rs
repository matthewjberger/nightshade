use crate::components::NodeKind;
use crate::data::title_of;
use crate::queries::compile_the_graph;
use crate::resources::{Chosen, Compiled};
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Res, ResMut};
use nightshade_platform::contract::window::Window;
use nightshade_ui_nodegraph::prelude::{NodePlace, WireFrom};

pub fn compile_the_shader(mut compiled: ResMut<Compiled>, storage: &mut Storage) {
    let wanted = compile_the_graph(storage).unwrap_or_default();
    compiled.nodes = storage.query_entities::<(NodePlace,)>().count();
    compiled.wires = storage.query_entities::<(WireFrom,)>().count();
    if compiled.source == wanted {
        return;
    }
    compiled.source = wanted;
    compiled.revision += 1;
}

pub fn name_the_window(
    compiled: Res<Compiled>,
    chosen: Res<Chosen>,
    mut window: ResMut<Window>,
    storage: &mut Storage,
) {
    let held = chosen
        .0
        .and_then(|node| storage.get::<NodeKind>(node))
        .map(|kind| title_of(kind.0))
        .unwrap_or("none");
    window.title = format!(
        "{} // {} nodes, {} wires // shader {} // chosen {held}",
        crate::theme::TITLE,
        compiled.nodes,
        compiled.wires,
        compiled.revision,
    );
}
