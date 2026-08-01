use crate::commands::let_go;
use crate::components::{WireFrom, WireTo};
use crate::data::{Holding, NodesLinked, NodesUnlinked};
use crate::queries::{already_linked, wire_under, wires_into, wires_out_of};
use crate::resources::{GraphGrip, NodeGraphSettings};
use nightshade::ecs::Storage;
use nightshade::ecs::system_param::{Commands, EventWriter, Res, ResMut};

pub fn settle_the_wires(
    settings: Res<NodeGraphSettings>,
    mut grip: ResMut<GraphGrip>,
    mut linked: EventWriter<NodesLinked>,
    mut unlinked: EventWriter<NodesUnlinked>,
    mut commands: Commands,
    storage: &mut Storage,
) {
    if grip.cutting {
        let socketed = match grip.over {
            Holding::Inlet(socket) => wires_into(storage, socket),
            Holding::Outlet(socket) => wires_out_of(storage, socket),
            _ => Vec::new(),
        };
        if socketed.is_empty() {
            if let Some((wire, from, to)) = wire_under(storage, &settings, grip.pointer) {
                commands.despawn(wire);
                unlinked.send(NodesUnlinked { from, to });
            }
        } else {
            for (wire, from, to) in socketed {
                commands.despawn(wire);
                unlinked.send(NodesUnlinked { from, to });
            }
        }
    }

    if !grip.letting_go {
        return;
    }

    let joined = match (grip.holding, grip.over) {
        (Holding::Outlet(from), Holding::Inlet(to)) => Some((from, to)),
        (Holding::Inlet(to), Holding::Outlet(from)) => Some((from, to)),
        _ => None,
    };

    if let Some((from, to)) = joined
        && from.node != to.node
        && !already_linked(storage, from, to)
    {
        commands.spawn((WireFrom(from), WireTo(to)));
        linked.send(NodesLinked { from, to });
    }

    let_go(&mut grip);
}
