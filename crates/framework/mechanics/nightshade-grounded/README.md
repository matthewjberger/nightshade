# nightshade-grounded

A body with a `Grounded` component stands on whatever the world calls ground. Each frame the mechanic reads the body's `LocalTransform`, asks `nightshade-ground` for the height under that spot, and writes the height back with the standing offset added. A game that carries hills, a heightmap or a block world answers the height question once and every grounded body follows it.

## Composing it

```rust
use nightshade_ground::Ground;
use nightshade_grounded::GroundedPlugin;

app.add_plugin(GroundedPlugin);

resources.res_mut::<Ground>().height_at = Some(hill_height);
```

`GroundedPlugin` inserts `Ground` when the program has not already, and registers `stand_on_the_ground` on `Stage::LateUpdate`, which is the one writer of the vertical value. The mechanic announces nothing and reads no other mechanic.

## Using it

```rust
use nightshade_grounded::Grounded;

storage.set(walker, Grounded(0.6));
```

`Grounded` holds the distance the body stands above the ground, so a tall figure carries a larger number than a coin. The mechanic skips a body whose spot has no height answer, which leaves that transform alone. A game that moves a body writes only the two flat axes of `LocalTransform` and lets the mechanic settle the third.

## Driving it

```
just example grounded
```

The arrow keys walk the slab over a field of hill columns and the mechanic settles its height on every step, Q and E lower and raise the `Grounded` distance so the body sinks into the hills or rides above them, and Space takes the height answer away so the ground stops answering at all. Walking past the rim of the field leaves the answer empty and the slab holds the last height it was given.
