# nightshade-portal

`nightshade-portal` draws one place through a surface standing in another. A pair of entities carrying a `Portal` plane and a `Link` to each other becomes two windows, and the crate spawns the eye cameras behind them, renders each into its own target, and publishes the orders the backend paints over the scene. It also carries the geometry a game needs around that, which is the matrix that carries a body from one plane to the other and the test for whether the body is touching a plane at all.

## Composing it

```rust
App::new()
    .add_plugin(DefaultPlugins)
    .add_plugin(nightshade_portal::prelude::PortalPlugin)
    .add_plugin(FourfoldPlugin)
    .run()
```

`PortalPlugin` inserts `PortalFrame` and `PortalSettings`, and registers two systems on `Stage::Update`, one that opens an eye chain behind every portal that lacks one and one that aims those eyes and fills the frame.

## Using it

```rust
let arch = commands.spawn((
    Portal(PortalPlane {
        center: gate.center,
        normal: gate.normal,
        up: Vec3::y(),
        width: ARCH_WIDTH,
        height: ARCH_HEIGHT,
    }),
    Rim(PORTAL_RIM),
));
commands.set(arch, Link(window));
commands.set(window, Link(arch));
portals.depth = 1;
```

A `Portal` holds the plane, which is a center, a normal, an up axis and a size, and `Link` names the portal on the other side. `Rim` tints the edge and `Layers` says which render layers the eye behind it can see. `PortalSettings` names the `viewer` entity the views are built from, the projection planes, the recursion `depth` and the `max_distance` past which a portal stops rendering, so a game sets `viewer` once at startup. For travel, `touching_portal` says whether a body has reached a plane and `portal_transform` gives the matrix that carries its position and orientation out of the other side.

## Taking it from another crate

The crate splits at a `contract` feature, which is how the backend reads the orders without linking the eye cameras.

```toml
nightshade-portal = { path = "../nightshade-portal", default-features = false, features = ["contract"] }
```

The contract carries `PortalPlane`, `PortalOrder` and the `PortalFrame` that gathers them.
