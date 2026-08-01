use crate::commands::place_a_node;
use crate::resources::Board;
use crate::theme;
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, EventReader, Query, Res, ResMut};
use nightshade_platform::contract::input::Input;
use nightshade_platform::prelude::KeyCode;
use nightshade_readout::Readout;
use nightshade_ui_nodegraph::prelude::{
    GraphGrip, Holding, NodeGraphSettings, NodeNote, NodePlace, NodesLinked, NodesUnlinked,
    WireFrom, cut_the_node, what_is_under,
};

pub fn take_the_orders(
    settings: Res<NodeGraphSettings>,
    grip: Res<GraphGrip>,
    input: Res<Input>,
    mut board: ResMut<Board>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    if !grip.inside {
        return;
    }
    if input.keyboard.just_pressed(KeyCode::KeyN) {
        let fresh = place_a_node(
            &mut commands,
            grip.pointer,
            theme::FRESH_TITLE,
            theme::FRESH_INLETS,
            theme::FRESH_OUTLETS,
            theme::FRESH_NOTE,
            theme::FRESH_TINT,
            false,
        );
        board.standing.push(fresh);
    }
    if input.keyboard.just_pressed(KeyCode::KeyX)
        && let Holding::Node(node) = what_is_under(storage, &settings, grip.pointer)
    {
        cut_the_node(&mut commands, storage, node);
        board.standing.retain(|standing| *standing != node);
    }
}

pub fn drag_the_field(grip: Res<GraphGrip>, mut board: ResMut<Board>, storage: &mut Storage) {
    let Holding::Note(node) = grip.holding else {
        return;
    };
    if grip.step.x == 0.0 {
        return;
    }
    let rate = (theme::SHAPE_HIGH - theme::SHAPE_LOW) * theme::SHAPE_SHARE;
    board.shaped = (board.shaped + grip.step.x * rate).clamp(theme::SHAPE_LOW, theme::SHAPE_HIGH);
    if let Some(note) = storage.get_mut::<NodeNote>(node) {
        note.0 = theme::note_of(board.shaped);
    }
}

pub fn count_the_changes(
    linked: EventReader<NodesLinked>,
    unlinked: EventReader<NodesUnlinked>,
    mut board: ResMut<Board>,
) {
    board.linked += linked.iter().count();
    board.cut += unlinked.iter().count();
}

pub fn show_the_board(
    grip: Res<GraphGrip>,
    board: Res<Board>,
    mut readout: ResMut<Readout>,
    mut places: Query<(&NodePlace,)>,
    mut wires: Query<(&WireFrom,)>,
) {
    let mut nodes = 0;
    let mut held = None;
    places.for_each_mut(|entity, (place,)| {
        nodes += 1;
        if matches!(grip.holding, Holding::Node(node) if node == entity) {
            held = Some(place.0);
        }
    });

    let mut strung = 0;
    let mut feeding = Vec::new();
    wires.for_each_mut(|_wire, (from,)| {
        strung += 1;
        if !feeding.contains(&from.0.node) {
            feeding.push(from.0.node);
        }
    });

    readout.notes.clear();
    readout.notes.push(format!(
        "{nodes} nodes, {strung} wires out of {} of them",
        feeding.len()
    ));
    readout
        .notes
        .push(format!("linked {}, cut {}", board.linked, board.cut));
    readout
        .notes
        .push(format!("the shaper carries {:.2}", board.shaped));
    readout.notes.push(match held {
        Some(place) => format!("holding a node at {:.0}, {:.0}", place.x, place.y),
        None => format!("pointer at {:.0}, {:.0}", grip.pointer.x, grip.pointer.y),
    });
}
