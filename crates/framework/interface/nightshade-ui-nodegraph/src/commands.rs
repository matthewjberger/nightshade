use crate::data::{Holding, wire_grips};
use crate::queries::wires_touching;
use crate::resources::{GraphGrip, GraphView, NodeGraphSettings};
use nalgebra_glm::{Vec2, Vec4, vec2};
use nightshade::ecs::system_param::Commands;
use nightshade::ecs::{Entity, Storage};
use nightshade_ui::prelude::{CanvasCommand, Rl, UiCanvasData, UiTreeBuilder};

pub fn spread_the_canvas(tree: &mut UiTreeBuilder<'_, '_>, view: &mut GraphView) -> Entity {
    let canvas = tree
        .add_node()
        .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
        .with_clip()
        .with_interaction()
        .entity();
    tree.commands().set(
        canvas,
        UiCanvasData {
            commands: Vec::new(),
            hit_test_enabled: true,
            command_bounds: Vec::new(),
        },
    );
    view.canvas = Some(canvas);
    canvas
}

pub fn ink_a_wire(
    storage: &mut Storage,
    canvas: Entity,
    from: Vec2,
    to: Vec2,
    settings: &NodeGraphSettings,
    tint: Vec4,
) {
    let (first, second) = wire_grips(from, to, settings.wire_reach);
    let Some(data) = storage.get_mut::<UiCanvasData>(canvas) else {
        return;
    };
    data.commands.push(CanvasCommand::CubicBezier {
        start: from,
        control1: first,
        control2: second,
        end: to,
        thickness: settings.wire_width,
        color: tint,
        id: None,
    });
}

pub fn cut_the_node(commands: &mut Commands, storage: &Storage, node: Entity) {
    for wire in wires_touching(storage, node) {
        commands.despawn(wire);
    }
    commands.despawn(node);
}

pub fn let_go(grip: &mut GraphGrip) {
    grip.holding = Holding::Nothing;
    grip.letting_go = false;
}
