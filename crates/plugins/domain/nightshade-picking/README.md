# nightshade-picking

`nightshade-picking` turns a point on the screen into a ray in the world and reports what that ray meets. It builds the ray from the active camera and the viewport, tests it against the bounding volume of every entity that has one, and hands back the entity, the distance and the world position of the hit. It also answers the flat question a strategy game asks, which is where the ray crosses the ground plane.

## Composing it

The crate carries no plugin. It is functions a system calls, so a program that picks composes nothing and names the crate where it asks.

## Using it

```rust
let intersection = nightshade_picking::screen_ray(
    view.window,
    view.viewport,
    view.active_camera,
    scene.storage.query_ref(),
    mouse_position,
)
.and_then(|ray| ray.intersect_ground_plane(0.0));
```

`screen_ray` takes the `Window`, the `Viewport`, the `ActiveCamera` and a query over the cameras, and returns the `PickingRay` under the given screen position. `intersect_ground_plane` and `intersect_plane` answer where that ray lands on a flat surface, which is enough for a grid game. To hit entities instead, pass a query over the bounding volumes and transforms to `pick_entities` with a `PickingOptions`, which filters by distance and skips invisible entities, or to `pick_closest_entity` for the nearest one alone. `pick_entities_in_frustum` narrows a list of entities to what a camera can see, and `ScreenArea` and `CameraView` are the two values every one of these calls resolves against.

## Taking it from another crate

The crate splits at a `contract` feature, which is how physics reports its trimesh ray results in the same types.

```toml
nightshade-picking = { path = "../nightshade-picking", default-features = false, features = ["contract"] }
```

The contract carries `PickingRay` with its plane intersections, `PickingResult` and `PickingOptions`, and nothing that reads the world.
