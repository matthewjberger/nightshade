use crate::components::{
    NodeField, NodeInlets, NodeNote, NodeOutlets, NodePlace, WireFrom, WireTo,
};
use crate::data::{Holding, Socket, WIRE_STEPS, step_distance, wire_grips, wire_point};
use crate::resources::NodeGraphSettings;
use nalgebra_glm::Vec2;
use nightshade::ecs::{Entity, Storage};
use nightshade_ui::prelude::Rect;

pub fn node_span(inlets: u32, outlets: u32, note: bool, settings: &NodeGraphSettings) -> Vec2 {
    let rows = (inlets + outlets) as f32;
    let noted = if note { settings.note_height } else { 0.0 };
    Vec2::new(
        settings.node_width,
        settings.header_height + rows * settings.row_height + noted + settings.node_tail,
    )
}

pub fn node_rect(
    place: Vec2,
    inlets: u32,
    outlets: u32,
    note: bool,
    settings: &NodeGraphSettings,
) -> Rect {
    Rect::from_min_max(place, place + node_span(inlets, outlets, note, settings))
}

pub fn inlet_place(place: Vec2, index: u32, outlets: u32, settings: &NodeGraphSettings) -> Vec2 {
    Vec2::new(place.x, socket_lift(place.y, outlets + index, settings))
}

pub fn outlet_place(place: Vec2, index: u32, settings: &NodeGraphSettings) -> Vec2 {
    Vec2::new(
        place.x + settings.node_width,
        socket_lift(place.y, index, settings),
    )
}

fn socket_lift(top: f32, row: u32, settings: &NodeGraphSettings) -> f32 {
    top + settings.header_height + settings.row_height * (row as f32 + 0.5)
}

pub fn note_place(place: Vec2, inlets: u32, outlets: u32, settings: &NodeGraphSettings) -> Vec2 {
    Vec2::new(
        place.x + settings.label_inset,
        place.y + settings.header_height + (inlets + outlets) as f32 * settings.row_height,
    )
}

pub fn note_rect(place: Vec2, inlets: u32, outlets: u32, settings: &NodeGraphSettings) -> Rect {
    let top = place.y + settings.header_height + (inlets + outlets) as f32 * settings.row_height;
    Rect::from_min_max(
        Vec2::new(place.x, top),
        Vec2::new(place.x + settings.node_width, top + settings.note_height),
    )
}

pub fn inlet_of(storage: &Storage, settings: &NodeGraphSettings, socket: Socket) -> Option<Vec2> {
    let place = storage.get::<NodePlace>(socket.node)?.0;
    let outlets = storage
        .get::<NodeOutlets>(socket.node)
        .map(|outlets| outlets.0)
        .unwrap_or_default();
    Some(inlet_place(place, socket.index, outlets, settings))
}

pub fn outlet_of(storage: &Storage, settings: &NodeGraphSettings, socket: Socket) -> Option<Vec2> {
    let place = storage.get::<NodePlace>(socket.node)?.0;
    Some(outlet_place(place, socket.index, settings))
}

pub fn wire_holds(from: Vec2, to: Vec2, settings: &NodeGraphSettings, point: Vec2) -> bool {
    let (first, second) = wire_grips(from, to, settings.wire_reach);
    let mut last = from;
    for step in 1..=WIRE_STEPS {
        let turn = step as f32 / WIRE_STEPS as f32;
        let next = wire_point(from, first, second, to, turn);
        if step_distance(point, last, next) <= settings.grab_slack {
            return true;
        }
        last = next;
    }
    false
}

pub fn what_is_under(storage: &Storage, settings: &NodeGraphSettings, point: Vec2) -> Holding {
    let mut found = Holding::Nothing;
    for (node, (place, inlets, outlets, note, field)) in storage
        .query_ref::<(
            &NodePlace,
            &NodeInlets,
            &NodeOutlets,
            Option<&NodeNote>,
            Option<&NodeField>,
        )>()
        .iter()
    {
        let fielded = note.is_some() && field.is_some_and(|field| field.0);
        let reach = settings.socket_radius + settings.grab_slack;
        for index in 0..inlets.0 {
            if (point - inlet_place(place.0, index, outlets.0, settings)).magnitude() <= reach {
                return Holding::Inlet(Socket { node, index });
            }
        }
        for index in 0..outlets.0 {
            if (point - outlet_place(place.0, index, settings)).magnitude() <= reach {
                return Holding::Outlet(Socket { node, index });
            }
        }
        if fielded && note_rect(place.0, inlets.0, outlets.0, settings).contains(point) {
            found = Holding::Note(node);
            continue;
        }
        if node_rect(place.0, inlets.0, outlets.0, note.is_some(), settings).contains(point) {
            found = Holding::Node(node);
        }
    }
    found
}

pub fn wire_under(
    storage: &Storage,
    settings: &NodeGraphSettings,
    point: Vec2,
) -> Option<(Entity, Socket, Socket)> {
    for (wire, (from, to)) in storage.query_ref::<(&WireFrom, &WireTo)>().iter() {
        let Some(tail) = outlet_of(storage, settings, from.0) else {
            continue;
        };
        let Some(head) = inlet_of(storage, settings, to.0) else {
            continue;
        };
        if wire_holds(tail, head, settings, point) {
            return Some((wire, from.0, to.0));
        }
    }
    None
}

pub fn wires_into(storage: &Storage, socket: Socket) -> Vec<(Entity, Socket, Socket)> {
    storage
        .query_ref::<(&WireFrom, &WireTo)>()
        .iter()
        .filter(|(_wire, (_from, to))| to.0 == socket)
        .map(|(wire, (from, to))| (wire, from.0, to.0))
        .collect()
}

pub fn wires_out_of(storage: &Storage, socket: Socket) -> Vec<(Entity, Socket, Socket)> {
    storage
        .query_ref::<(&WireFrom, &WireTo)>()
        .iter()
        .filter(|(_wire, (from, _to))| from.0 == socket)
        .map(|(wire, (from, to))| (wire, from.0, to.0))
        .collect()
}

pub fn wires_touching(storage: &Storage, node: Entity) -> Vec<Entity> {
    storage
        .query_ref::<(&WireFrom, &WireTo)>()
        .iter()
        .filter(|(_wire, (from, to))| from.0.node == node || to.0.node == node)
        .map(|(wire, _ends)| wire)
        .collect()
}

pub fn already_linked(storage: &Storage, from: Socket, to: Socket) -> bool {
    storage
        .query_ref::<(&WireFrom, &WireTo)>()
        .iter()
        .any(|(_wire, (tail, head))| tail.0 == from && head.0 == to)
}
