# nightshade-hover

A pickup, a coin or a rune sits above a fixed spot and rides up and down on a sine. The mechanic holds the spot in `Perch`, the height of the ride in `Bob`, the offset into the wave in `Phase`, and a turn rate in `Spin`, and writes the body's `LocalTransform` from them each frame. Giving each body its own phase keeps a field of them from moving as one.

## Composing it

```rust
use nightshade_hover::{HoverPlugin, HoverSettings};

app.add_plugin(HoverPlugin {
    settings: HoverSettings { rate: 1.2 },
});
```

`HoverPlugin` inserts `HoverSettings`, which carries the one rate the whole field bobs at, and registers `hold_them_up` on `Stage::LateUpdate`, where it is the one writer of those transforms. It announces nothing.

## Using it

```rust
use nightshade_hover::queries::lift;
use nightshade_hover::{Bob, Perch, Phase, Spin};

storage.set(coin, Perch(vec3(2.0, 1.0, -3.0)));
storage.set(coin, Bob(0.25));
storage.set(coin, Phase(index as f32 * 0.7));
storage.set(coin, Spin(1.5));

let up = lift(&bob, &phase, &settings, clock);
```

A body needs all four components, because the mechanic's query names all four. `Perch` is where the body hangs, so a game that moves a hovering thing writes `Perch` rather than the transform the mechanic overwrites. `Spin` of zero leaves the rotation alone, and `queries::lift` reports the same offset the mechanic applies for a game that wants to draw or measure it.

## Driving it

```
just example hover
```

The left and right arrows pick a coin out of the row and it steps forward on its perch, the up and down arrows drive that perch higher and lower, Z and X narrow and widen its `Bob`, Space turns its `Spin` on and off, and Q and E change the rate every coin bobs at. The readout gives the lift `queries::lift` reports for the chosen coin beside the height the mechanic actually put it at.
