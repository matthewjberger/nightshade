# nightshade-ui-nodegraph

A node graph on one retained canvas. A node is a row of components on an ordinary entity, a wire is an entity with the two sockets it joins, and the crate holds no graph object of its own: the world is the graph. Every frame the crate reads the rows, paints the bodies, the sockets and the wires into the canvas, and turns what the pointer does over that canvas into rows again.

## Composing it

```rust
app.add_plugin(nightshade_ui::UiPlugin::new());
app.add_plugin(nightshade_ui_nodegraph::NodeGraphPlugin::default());
```

The plugin inserts `NodeGraphSettings`, which carries every measure and tint the paint pass reads, and a game that wants its own look hands its own settings to the plugin. It also inserts `GraphView`, which holds the canvas and the pan, and `GraphGrip`, which holds what the pointer is over and what it has hold of. Four systems run over them: `take_the_grip` on `Stage::First` turns the pointer into intent, `move_what_is_held` on `Stage::Update` drags a node or pans the view, `settle_the_wires` on `Stage::LateUpdate` strings a new wire or cuts an old one, and `paint_the_graph` follows it with the canvas commands.

## Giving it a canvas

```rust
fn spread_the_board(mut view: ResMut<GraphView>, mut build: UiBuild) {
    let mut tree = UiTreeBuilder::new(build.context());
    spread_the_canvas(&mut tree, &mut view);
    tree.finish();
}
```

`spread_the_canvas` adds an interactive canvas that fills its parent and writes it into the view. A game that wants the graph inside a panel adds it under that panel instead.

## Filling it

A node is five components and a wire is two, and four more components give a node its detail.

```rust
let source = commands.spawn((
    NodePlace(vec2(120.0, 140.0)),
    NodeTitle("Source".to_string()),
    NodeInlets(0),
    NodeOutlets(2),
    NodeTint(vec4(0.28, 0.46, 0.66, 1.0)),
));
commands.spawn((
    WireFrom(Socket { node: source, index: 0 }),
    WireTo(Socket { node: shaper, index: 0 }),
));
```

`NodePlace` is where the top left corner of the body sits in graph space, and the crate writes it while a person drags the node. The socket places follow from the counts, so nothing stores them: the outlets take the rows under the header and the inlets take the rows under those, the way a shading graph reads.

`NodeInletNames`, `NodeOutletNames`, `NodeNote` and `NodeField` are what make a node legible. The names sit beside their sockets, the inlets to the right of theirs and the outlets to the left of theirs, and the note is one line below the last row for what the node carries or computes. A node without them still draws, as a titled body with bare sockets.

`NodeField(true)` says the note row is a value a person sets in the node. Such a row draws as an inset field, lights when the pointer is over it and shows a grip arrow at each end, and while it is held the grip reports `Holding::Note` with the step the pointer took, so a game reads that step and changes whatever its own value is. A row without the component is plain text and a drag over it moves the node, the way the rest of the body does. The crate never learns what the number means. `cut_the_node` takes a node away with every wire that touched it, and `wires_touching`, `wires_into`, `wires_out_of`, `wire_under` and `what_is_under` report what the pointer has found without changing anything. A right click drops what is under it: every wire on a socket when the pointer is on one, and the one wire beneath the pointer otherwise.

## Hearing it

`NodesLinked` and `NodesUnlinked` go out on the bus when a wire settles, so a game that compiles the graph rebuilds only when it hears one.

## Driving it

```
just example nodegraph
```

Drag a node by its body and the wires follow. Drag from a socket to a socket of the other kind to string a new wire, and let go over empty space to drop it. Drag the background to pan. Right click a wire to cut it. The `N` key places a node under the pointer and the `X` key takes the one under the pointer away. The readout gives the count of nodes and wires, how many wires have been strung and cut, and where the held node sits.
