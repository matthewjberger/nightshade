# nightshade-noise

A field of numbers that looks unplanned and comes back the same every time. A seed and a coordinate go in and the same value always comes out, so a world generated from it rebuilds identically on the next run and across machines. Terrain height, scatter density, and the per-chunk seed a generator branches from all read from here.

## Composing it

This crate carries no plugin and no resource. A game names it in its own `Cargo.toml` and calls the functions directly.

```toml
nightshade-noise = { workspace = true }
```

A caller holds its own seed and decides what each coordinate means.

## Using it

```rust
use nightshade_noise::{hash, hash_two, layered, ridged, seed_of, unit, value};

let scrambled = hash(index);
let mixed = hash_two(across, down, seed);
let flat = unit(across, down, seed);
let smooth = value(across as f32 * 0.05, down as f32 * 0.05, seed);
let hills = layered(across * 0.02, down * 0.02, seed, 4, 0.5);
let ridges = ridged(across * 0.02, down * 0.02, seed, 4, 0.5);
let chunk_seed = seed_of(chunk_across, chunk_down);
```

`hash` and `hash_two` scramble integers and are the base every other function is built on. `unit` turns an integer coordinate into a value between zero and one, and `value` interpolates between four of those with a smooth falloff, so it varies continuously as the coordinate moves. `layered` sums several octaves of `value`, doubling the frequency and multiplying the weight by `fade` each time, and normalizes the result back into zero to one. `ridged` folds that sum around its midpoint so the peaks come out as sharp lines, and `seed_of` packs a pair of coordinates into a `u64` a chunk generator can seed its own roll from.

## Driving it

```
just example noise
```

The arrow keys walk the marker over the field, and it rides at whatever height `layered` reports beneath it. R scrambles the seed through `hash` and Q and E take a layer off the sum or add one, and every column is raised again to its new height and retinted, so the whole field rebuilds under the marker while it stands there. The readout gives the column count, the layer count, the seed and the height under the marker.
