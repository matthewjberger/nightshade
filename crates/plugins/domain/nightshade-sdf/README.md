# nightshade-sdf

A signed distance field is built from edits, each one a shape combined with what is already there. An entity that carries `SdfShape`, `SdfOperation`, `SdfBlend`, `SdfTint` and `SdfOrder` beside a transform is gathered every frame into `SdfFrame`, which the wgpu backend reads and raymarches, and the same edits are meshed on a worker crew into `SdfCollisionMesh` chunks a physics body can stand on. Every shape but the terrain is placed by its transform, and the terrain is evaluated in world space. The crate names no GPU API.

## Composing it

```rust
app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
    .add_plugin(nightshade_ui::UiPlugin::new())
    .add_plugin(nightshade_sdf::SdfPlugin)
    .add_plugin(nightshade_physics::PhysicsPlugin)
    .add_plugin(SdfDemoPlugin)
    .run()
```

`SdfPlugin` inserts `SdfFrame`, `SdfClipmap`, `SdfDebugView` and `SdfCollision`, then registers `systems::gather::gather_edits_system` and `systems::collision::sdf_collision_system` on `Stage::Render`.

## Using it

```rust
use nightshade_sdf::contract::{SdfBlend, SdfOperation, SdfOrder, SdfShape, SdfTint};

let local_transform = LocalTransform {
    translation,
    ..Default::default()
};
let edit = commands.spawn((
    Name(name.to_string()),
    local_transform,
    GlobalTransform(local_transform.as_matrix()),
    shape,
));
commands.set(edit, SdfOperation::Union);
commands.set(edit, SdfBlend(blend));
commands.set(edit, SdfTint(tint));
commands.set(edit, SdfOrder(order));
```

`SdfShape` is the sphere, cube, capsule, torus, cylinder, octahedron or terrain the edit contributes, and `SdfOperation` says whether it adds, cuts or intersects. `SdfBlend` softens the seam, `SdfTint` colors the surface, `SdfRoughness` and `SdfMetal` say how it reflects the sun and the sky, and `SdfOrder` fixes the sequence the edits are folded in. `SdfCollides` takes an edit out of the collision meshing while leaving it on screen, and `SdfCollisionFocus` on an entity marks where the chunks are built around. `SdfClipmap` holds the level count, the finest cell and the bricks per side the field is sampled at.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-sdf = { path = "../nightshade-sdf", default-features = false, features = ["contract"] }
```

The contract carries the edit components, the field evaluation in `contract::field`, and the render data a backend draws from: `SdfFrame`, `SdfEditInstance`, `SdfClipmap`, `SdfChunkKey`, `SdfCollisionMesh` and `SdfDebugView`. `nightshade-wgpu-sdf` takes it that way.
