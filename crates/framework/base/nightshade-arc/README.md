# nightshade-arc

Whether a target stands inside the arc a swing covers on the play plane. A melee hit, a cone of vision, and a shout that reaches only what is ahead all ask the same question, and this crate answers it with one call. It measures on the plane the game plays on, so a target overhead or underfoot is judged by where it stands rather than how high it is.

## Composing it

This crate carries no plugin and no resource. A game names it in its own `Cargo.toml` and calls the functions directly.

```toml
nightshade-arc = { workspace = true }
```

A caller holds the `Plane` the game plays on, the reach of the swing, and the half angle that gives the arc its width.

## Using it

```rust
use nalgebra_glm::Vec3;
use nightshade_arc::queries::{facing_from_yaw, in_arc};
use nightshade_plane::Plane;

let facing = facing_from_yaw(yaw);
let struck = in_arc(
    Plane::Ground,
    attacker.translation,
    facing,
    target.translation,
    reach,
    half_angle,
);
```

`in_arc` flattens both points onto the play plane, rejects anything past `reach` or standing exactly on the attacker, and then compares the angle between the facing direction and the direction to the target against `half_angle`. A half angle of `std::f32::consts::FRAC_PI_4` gives a ninety degree cone. A facing vector with no length on the play plane answers false rather than picking an arbitrary direction. `facing_from_yaw` builds the forward vector a yaw points along, so a body that stores only its turn does not have to derive one.

## Driving it

```
just example arc
```

The left and right arrows turn the swinger and the up and down arrows walk it across the floor, Q and E narrow and widen the half angle, and Z and X shorten and lengthen the reach. Every target `in_arc` answers true for lifts while it is covered, so the cone is drawn out by what rises, and the readout gives the count inside it beside the half angle, the reach and the yaw.
