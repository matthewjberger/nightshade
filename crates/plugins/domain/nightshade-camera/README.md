# nightshade-camera

nightshade-camera holds what a camera is. `Camera` carries a projection and optional smoothing, the projection is perspective or orthographic, and the render target, the projection override and the viewport shading sit beside them as their own components. Each frame the crate turns the camera entities into the `RenderViews` the backend draws from.

## Composing it

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .run()
```

`CameraPlugin` is a member of every backend's `DefaultPlugins`, so a program that composes the group already has it, and `app.add_plugin(nightshade_camera::CameraPlugin)` is the line a program that composes its own set writes. The plugin inserts `ActiveCamera` and `RenderViews`, then registers `publish_render_views` on `Stage::Render`.

## Using it

```rust
use nightshade_camera::prelude::{
    ActiveCamera, Camera, PerspectiveCamera, Projection, Smoothing, spawn_camera,
};

let camera = spawn_camera(&mut commands, Vec3::new(0.0, 4.0, 12.0), "Main Camera".to_string());
commands.set(
    camera,
    Camera {
        projection: Projection::Perspective(PerspectiveCamera {
            aspect_ratio: None,
            y_fov_rad: 60.0_f32.to_radians(),
            z_near: 0.1,
            z_far: Some(400.0),
        }),
        smoothing: Some(Smoothing::default()),
    },
);
active_camera.0 = Some(camera);
```

`spawn_camera` puts a perspective camera at a place with a name, and `spawn_ortho_camera` does the same for a flat one. `ActiveCamera` holds the entity the frame renders from, so a system takes `ResMut<ActiveCamera>` and writes its one field. An `aspect_ratio` of `None` fits the projection to the viewport, and `Projection::matrix_with_aspect` is the path a consumer takes to build the matrix. The `queries` module reports the matrices, the frustum and the aspect ratios from the same rows.

## Taking it from another crate

The crate splits at a `contract` feature, so a crate that only names the types writes `nightshade-camera = { path = "../nightshade-camera", default-features = false, features = ["contract"] }`. The contract carries the `Camera` component, the two projections, the smoothing, the render target, the projection override, the viewport shading and the `ActiveCamera` resource, which is what `nightshade-camera-controller`, `nightshade-openxr` and `nightshade-picking` read.
