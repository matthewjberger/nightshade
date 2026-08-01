# nightshade-ease

The curves a value follows when it moves toward another one. It holds seven plain functions over `f32` and keeps no state, so a game reaches for it wherever a number has to arrive somewhere smoothly rather than jumping. A camera settling behind a car, a bar draining, and a door swinging open all read the same handful of shapes from here.

## Composing it

This crate carries no plugin and no resource. A game names it in its own `Cargo.toml` and calls the functions directly.

```toml
nightshade-ease = { workspace = true }
```

A caller holds nothing. Every function takes its inputs and returns the answer.

## Using it

```rust
use nightshade_ease::{approach, damp, lerp, remap, share, smooth_step, smoother_step};

let straight = lerp(from, to, turn);
let eased = lerp(from, to, smooth_step(turn));
let gentler = lerp(from, to, smoother_step(turn));
let walked = approach(here, to, speed * delta);
let settled = damp(here, to, rate, delta);
let part = share(rate, delta);
let loudness = remap(distance, 0.0, 20.0, 1.0, 0.0);
```

`lerp` is the straight line between two values. `smooth_step` and `smoother_step` bend an amount already between zero and one, so they are passed through `lerp` rather than used on their own. `approach` moves by a fixed step and lands exactly on the target, while `damp` closes a fraction of the remaining gap each second and stays frame rate independent. `share` clamps a rate times a delta into zero to one, and `remap` carries a value from one range into another with the result clamped.

## Driving it

```
just example ease
```

Hold the left and right arrows to walk the turn between zero and one, and press R or F to send it straight to an end. The first three runners sit where `lerp`, `smooth_step` and `smoother_step` put them for that turn, the fourth walks after the first at a fixed step and the fifth settles toward it, and the readout gives the turn, the two eased amounts and how far along the track the two chasers have reached.
