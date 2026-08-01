# nightshade-random

A seeded generator and the shapes a game asks it for. `Random` wraps one `fastrand::Rng`, so a program that seeds it once replays the same scatter, the same coin flips, and the same picks every run. The functions over it cover the asks a game actually makes rather than raw bits.

## Composing it

This crate carries no plugin. A game names it in its own `Cargo.toml` and inserts `Random` with the seed it wants.

```rust
app.insert_resource(Random::seeded(0x1234_5678));
```

`Random::default()` seeds with zero. A caller holds the resource and takes it mutably, because every draw advances the generator.

## Using it

```rust
use nightshade_random::Random;
use nightshade_random::commands::{
    annulus, between, chance, count, direction, disc, index, pick, ring, shuffle, span,
};

let random = resources.res_mut::<Random>();
let heads = chance(random, 0.5);
let angle = between(random, 0.0, std::f32::consts::TAU);
let drift = span(random, 2.0);
let many = count(random, 3, 8);
let picked = index(random, names.len());
let edge = ring(random, radius);
let inside = disc(random, radius);
let heading = direction(random);
```

`chance` answers a likelihood between zero and one, `between` draws a float in a range, and `span` is that range centered on zero. `count` draws a whole number from a low bound up to a high one and returns the low bound when the range is empty, and `index` picks a position in a list of a given length, answering zero for an empty one. `ring` lands on the edge of a circle and `disc` lands anywhere inside it with the points spread evenly rather than crowding the middle. `direction` returns a unit `Vec3` pointing anywhere on a sphere.

## Driving it

```
just example random
```

Space draws a fresh scatter, the blue grains from `disc` and the gold ones from `ring`, and R seeds the generator again so the same arrangement comes back exactly as it was. Q and E narrow and widen the radius the two shapes are drawn at, and the readout gives how many of the coin flips came up heads and the radius in force.
