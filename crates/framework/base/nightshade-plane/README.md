# nightshade-plane

The plane a game plays on, and the measure across it. `Plane::Ground` is a world seen from above, where the play runs over X and Z and Y is height, and `Plane::Side` is a world seen from the side, where the play runs over X and Y and Z is depth. A mechanic that takes a `Plane` measures distance, flattens a point, and steps a body without knowing which of the two the game picked.

## Composing it

This crate carries no plugin. A game names it in its own `Cargo.toml` and inserts `PlayPlane` when it wants one answer for the whole program.

```rust
app.insert_resource(PlayPlane(Plane::Ground));
```

A caller holds the `Plane` value itself, either read out of `PlayPlane` or passed along by whatever mechanic is asking.

## Using it

```rust
use nalgebra_glm::Vec3;
use nightshade_plane::{Plane, PlayPlane};
use nightshade_plane::commands::move_across;
use nightshade_plane::queries::{across, flatten, lift, raise};

let plane = resources.res::<PlayPlane>().0;
let flat = flatten(plane, transform.translation);
let back = raise(plane, flat);
let reach = across(plane, from, to);
let height = lift(plane, transform.translation);
move_across(plane, &mut transform.translation, step);
```

`flatten` drops a world point onto the two axes the play runs over and `raise` puts a flat point back into the world with zero on the third. `across` is the distance between two points measured only over the play axes, so a target overhead reads as close. `lift` reports the axis the play does not run over, which is height on the ground plane and depth on the side plane. `move_across` adds a step to a position and leaves that third axis untouched, so gravity or a jump stays the caller's to write.

## Driving it

```
just example plane
```

The arrow keys push the walker with `move_across` and Space swaps `PlayPlane` between the two planes. The same four keys slide the walker over the floor on the ground plane and lift it into the air on the side plane, it turns from blue to gold when the plane changes, and the readout gives the plane, the flattened spot, the distance across from the middle and the axis the play leaves alone.
