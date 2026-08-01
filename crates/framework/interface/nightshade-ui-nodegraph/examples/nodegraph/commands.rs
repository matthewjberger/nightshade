use crate::theme;
use nalgebra_glm::{Vec2, Vec4};
use nightshade::ecs::system_param::Commands;
use nightshade::prelude::Entity;
use nightshade_ui_nodegraph::prelude::{
    NodeField, NodeInletNames, NodeInlets, NodeNote, NodeOutletNames, NodeOutlets, NodePlace,
    NodeTint, NodeTitle, Socket, WireFrom, WireTo,
};

fn owned(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

pub fn place_a_node(
    commands: &mut Commands,
    place: Vec2,
    title: &str,
    inlets: &[&str],
    outlets: &[&str],
    note: &str,
    tint: Vec4,
    fielded: bool,
) -> Entity {
    let node = commands.spawn((
        NodePlace(place),
        NodeTitle(title.to_string()),
        NodeInlets(inlets.len() as u32),
        NodeOutlets(outlets.len() as u32),
        NodeTint(tint),
    ));
    commands.set(node, NodeInletNames(owned(inlets)));
    commands.set(node, NodeOutletNames(owned(outlets)));
    commands.set(node, NodeNote(note.to_string()));
    commands.set(node, NodeField(fielded));
    node
}

pub fn string_a_wire(commands: &mut Commands, from: Socket, to: Socket) {
    commands.spawn((WireFrom(from), WireTo(to)));
}

pub fn stand_the_graph(commands: &mut Commands) -> Vec<Entity> {
    let source = place_a_node(
        commands,
        theme::SOURCE_PLACE,
        "Source",
        &[],
        &["left", "right"],
        "two streams",
        theme::SOURCE_TINT,
        false,
    );
    let shaper = place_a_node(
        commands,
        theme::SHAPER_PLACE,
        "Shaper",
        &["in"],
        &["out"],
        &theme::note_of(theme::SHAPE_START),
        theme::SHAPER_TINT,
        true,
    );
    let mixer = place_a_node(
        commands,
        theme::MIXER_PLACE,
        "Mixer",
        &["a", "b"],
        &["out"],
        "folds two into one",
        theme::MIXER_TINT,
        false,
    );
    let sink = place_a_node(
        commands,
        theme::SINK_PLACE,
        "Sink",
        &["a", "b"],
        &[],
        "where it ends",
        theme::SINK_TINT,
        false,
    );

    string_a_wire(
        commands,
        Socket {
            node: source,
            index: 0,
        },
        Socket {
            node: shaper,
            index: 0,
        },
    );
    string_a_wire(
        commands,
        Socket {
            node: source,
            index: 1,
        },
        Socket {
            node: mixer,
            index: 0,
        },
    );
    string_a_wire(
        commands,
        Socket {
            node: shaper,
            index: 0,
        },
        Socket {
            node: sink,
            index: 0,
        },
    );
    string_a_wire(
        commands,
        Socket {
            node: mixer,
            index: 0,
        },
        Socket {
            node: sink,
            index: 1,
        },
    );

    vec![source, shaper, mixer, sink]
}
