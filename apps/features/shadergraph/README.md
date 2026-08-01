# shadergraph

A node graph that is the shader. The nodes are shading operations, the wires carry values between them, and every time the graph changes the app writes WGSL from it, hands the source to a pass, and that pass draws the picture. Nothing is precompiled: the source is built from the rows in the world.

## What is on the screen

Two docked panels hold the interface. `Nodes` on the left runs in three sections: the preset dropdown, the picture button, and the node families a person places from, with the button that writes the shader out at the foot. `Graph` along the bottom holds the editor, so the nodes sit on a panel of their own rather than over the picture. What the two panels leave is the preview, and the pass draws there and nowhere else: it reads the reserved edges from the dock each frame and sets its viewport and scissor from them. Drag a panel edge and the preview follows. The window title gives the node count, the wire count, the shader revision and the kind of the chosen node.

The node families are a scrolling shelf of collapsing sections, `Sources`, `Space`, `Arithmetic`, `Shaping`, `Pattern`, `Color` and `Result`, so a person reads one family at a time rather than every kind at once.

Every value is set in the node that carries it. A node that carries a number shows it in an inset field at the foot of its body, with the name of the value and the number, and dragging that field across sets it. The panel carries no slider, so there is one place a value lives.

## Pictures

A `Picture` node samples one of eight slots. Two are filled at startup from pictures the repository already carries, `awesomeface.png` and `cobblestone.png`, so the picture presets work the moment the app opens. The `Open a picture` button plucks a file from the disk, decodes it through the engine's own decoder, and puts it in the next of the six free slots. Clicking the field on a `Picture` node opens the same dialog and lands the file in that node's slot. If a `Picture` node is the chosen one the file lands there too, and if not the app places a fresh node for it.

The slot is baked into the source as a literal, so nothing is indexed at run time and the shader needs no device feature. An empty slot holds a single white texel, which is why a graph that samples nothing still compiles and draws. The sampler repeats, so a uv beyond one tiles.

The file dialog is desktop only. On the web and on android the button reports that there is no dialog there.

## The kinds

`UV` and `Middle` give the pixel place, from the corner and from the center, `Time` runs, `Value` is a constant a person sets, and `Sweep` runs an eased front out and back over a period. `Across` and `Down` take one axis of a value, `Length` and `Polar` measure it from the center, and `Rotate` turns it. `Add`, `Subtract`, `Multiply`, `Min`, `Max` and `Power` are the arithmetic, `Sine`, `Cosine`, `Fract`, `Floor`, `Abs`, `Step`, `Invert` and `Smooth` shape one value, and `Luma` says how bright a color is. `Noise` is fractal noise over the xy of its input, `Lattice` draws the cell edges of a grid, `Picture` samples a slot, `Mix` blends two values by a third, `Palette` turns one number into a color, `Seam` is a hot band around zero, and `Output` is the color the pass writes. Every wire carries a `vec4`, so a scalar broadcasts and a color passes through unchanged.

## The presets

The dropdown carries twenty-one graphs. Fourteen are made of math alone: `Noise field`, `Drifting noise`, `Rings`, `Ripple`, `Stripes`, `Plasma`, `Warp`, `Vortex`, `Sun`, `Bands`, `Marble`, `Grid`, `Checker` and `Clouds`. Six read a picture: `Picture`, `Picture warp`, `Picture ripple`, `Picture kaleidoscope`, `Picture edges` and `Stone tiles`. `Materialize` is the seventh picture graph and the largest: a wobbled front sweeps out and back over the frame, a wire lattice stands ahead of it, the picture phases in behind it, and a hot seam rides the boundary. Each one is a table of kinds and wires in `data.rs` rather than code, so picking one clears the graph and spawns those rows. From there it is an ordinary graph: move the nodes, cut a wire, change a value.

## How it compiles

`compile_the_graph` finds the `Output` node, walks the wires backwards, and emits one `let` for each node in the order its inputs demand. A missing input reads as zero and a cycle stops at zero, so no graph can fail to compile. The result is a fragment body. `GraphPass` puts it between the prelude in `shaders/frame.wgsl` and its closing brace, compiles the module, and rebuilds the pipeline. It only does that when the revision moves or the target format changes.

## Driving it

```
just run shadergraph
```

Pick a preset from the dropdown, or open a family and click a name to place that node. Drag the background to pan, and the `X` key takes the node under the pointer away.

Every node answers the pointer. Drag its body to move it. Drag from a socket to a socket of the other kind to wire them, right click a socket to drop every wire on it, and right click a wire to cut that one.

A node that carries a number shows an inset field at the foot of its body, and a drag across that field sets the number: `Value` carries the number itself, `Sweep` the length of its cycle, `Noise` the scale of its field, `Lattice` its cell count, `Seam` its width, and `Palette` how far the colors turn. The field lights and shows a grip arrow at each end while the pointer is on it. A `Picture` node carries a slot, so dragging its field steps to the next picture that is loaded and clicking it opens a file. A node that carries no number shows its note as plain text, and a drag over that text moves the node.
