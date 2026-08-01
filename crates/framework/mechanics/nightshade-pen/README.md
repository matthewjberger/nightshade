# nightshade-pen

A body with a `Pen` component is held inside a rectangle centered on the origin. Each frame the mechanic flattens the body's position onto the play plane, clamps it into the half extents the pen carries, and moves it back by the difference. It works the same whether the game plays on the ground or from the side, because `nightshade-plane` decides which two axes are flat.

## Composing it

```rust
use nightshade_pen::PenPlugin;

app.add_plugin(PenPlugin);
```

`PenPlugin` inserts `PlayPlane` when the program has not already, and registers `keep_them_inside` on `Stage::LateUpdate`, which runs after the movers have written the frame's positions. It announces nothing.

## Using it

```rust
use nightshade_pen::Pen;
use nightshade_pen::queries::{held, inside};

storage.set(walker, Pen(vec2(12.0, 8.0)));

let still_in = inside(vec2(12.0, 8.0), wanted);
let clamped = held(vec2(12.0, 8.0), wanted);
```

`Pen` holds the half extents, so a pen of twelve by eight reaches twelve either side of the origin and eight the other way. A body without `Pen` is untouched, which is how a game keeps one thing fenced and lets another leave. `queries::inside` reports whether a flat spot is already within a pen and `held` reports the clamped spot, so a game that wants to refuse a move rather than clamp it can ask before it writes.

## Driving it

```
just example pen
```

The arrow keys shove both slabs the same way, the lower one fenced and the upper one free, and R puts them back over the middle. The fenced slab stops at the wall while the free one carries on, and the readout gives the pen, the spot the free slab reached and the spot the fenced one was held at.
