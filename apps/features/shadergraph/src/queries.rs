use crate::components::{NodeKind, NodeSlot, NodeValue};
use crate::data::{EMPTY, Kind, inlets_of, wgsl_of};
use nightshade::ecs::{Entity, Storage};
use nightshade_ui_nodegraph::prelude::{WireFrom, WireTo};

struct Feed {
    into: Entity,
    inlet: u32,
    from: Entity,
}

pub fn the_output(storage: &Storage) -> Option<Entity> {
    storage
        .query_ref::<(&NodeKind,)>()
        .iter()
        .find(|(_node, (kind,))| kind.0 == Kind::Output)
        .map(|(node, _kind)| node)
}

pub fn compile_the_graph(storage: &Storage) -> Option<String> {
    let output = the_output(storage)?;
    let feeds: Vec<Feed> = storage
        .query_ref::<(&WireFrom, &WireTo)>()
        .iter()
        .map(|(_wire, (from, to))| Feed {
            into: to.0.node,
            inlet: to.0.index,
            from: from.0.node,
        })
        .collect();

    let mut lines = Vec::new();
    let mut done = Vec::new();
    let mut open = Vec::new();
    let last = emit(storage, &feeds, output, &mut done, &mut open, &mut lines);
    lines.push(format!("    return {last};"));
    Some(lines.join("\n"))
}

fn emit(
    storage: &Storage,
    feeds: &[Feed],
    node: Entity,
    done: &mut Vec<Entity>,
    open: &mut Vec<Entity>,
    lines: &mut Vec<String>,
) -> String {
    let name = format!("v{}_{}", node.id, node.generation);
    if done.contains(&node) {
        return name;
    }
    if open.contains(&node) {
        return EMPTY.to_string();
    }
    let Some(kind) = storage.get::<NodeKind>(node).map(|kind| kind.0) else {
        return EMPTY.to_string();
    };
    let value = storage
        .get::<NodeValue>(node)
        .map(|value| value.0)
        .unwrap_or_default();
    let slot = storage
        .get::<NodeSlot>(node)
        .map(|slot| slot.0)
        .unwrap_or_default();

    open.push(node);
    let mut inputs = Vec::new();
    for inlet in 0..inlets_of(kind) {
        let fed = feeds
            .iter()
            .find(|feed| feed.into == node && feed.inlet == inlet);
        match fed {
            Some(feed) => inputs.push(emit(storage, feeds, feed.from, done, open, lines)),
            None => inputs.push(EMPTY.to_string()),
        }
    }
    open.pop();

    lines.push(format!(
        "    let {name} = {};",
        wgsl_of(kind, &inputs, value, slot)
    ));
    done.push(node);
    name
}
