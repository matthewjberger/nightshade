# nightshade-ground

The question a mechanic asks a continuous world about height and solidity. `Ground` is a table of two function pointers the game fills, so a mechanic that keeps a body standing on the floor asks how high the ground is at a spot and whether a point is inside something solid without knowing whether the answer comes from a heightfield, a voxel world, or a flat plane.

## Composing it

This crate carries no plugin. A game names it in its own `Cargo.toml` and inserts the resource before anything asks it a question.

```rust
app.insert_resource(Ground::default());
```

A caller holds nothing. The answers live in the `Ground` resource, and every query takes that resource and the rows it reads.

## Using it

```rust
use nalgebra_glm::Vec3;
use nightshade_ground::Ground;
use nightshade_ground::commands::{answer_height_with, answer_solid_with};
use nightshade_ground::queries::{answers_height, answers_solid, height_at, solid_at};

let ground = resources.res_mut::<Ground>();
answer_height_with(ground, hill_height);
answer_solid_with(ground, solid_here);

let high = height_at(&ground, storage, spot).unwrap_or(0.0);
let blocked = solid_at(&resources, Vec3::new(spot.x, high + 0.25, spot.y));
```

Each answer is a plain function pointer taking the rows, so the game's own function reads the components that hold its terrain, and a caller names `Res<Ground>` rather than the whole resource map. `height_at` returns `None` when the game set no answer or when the answer itself has nothing at that spot, which is how a mechanic tells a hole from a floor at zero. `solid_at` answers false when no answer is set, so a world that never fills it in is open everywhere. `answers_height` and `answers_solid` report whether the game has filled a slot, which lets a mechanic skip work in a world that cannot answer.

## Driving it

```
just example ground
```

The arrow keys push the red probe over the hills, and it rides at whatever height the game's own answer reports beneath it while the ring of resting probes holds its own spots. The probe turns green the moment `solid_at` answers true for the point it occupies, which happens where it crosses the block in the middle, and the readout gives the height it rides at, the spot it stands over, whether it is inside anything solid and how many probes are.
