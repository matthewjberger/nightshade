# nightshade-camera-controller

nightshade-camera-controller moves a camera from input. It carries the pan orbit camera, the third person camera and the fly camera, and reads the mouse, the keyboard, the gamepad and the touch screen each frame to turn them. It also carries a collision hook a game fills so a following camera stops at the world rather than passing through it.

## Composing it

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(nightshade_camera_controller::CameraControllerPlugin)
    .run()
```

`CameraControllerPlugin` inserts `CameraCollisionHook` and registers its systems on `Stage::FrameUpdate`: the look camera, the touch fly camera, the keyboard movement, the gamepad fly camera, the four pan orbit systems and the pending move drain.

## Using it

```rust
use nightshade_camera_controller::commands::spawn_pan_orbit_camera_at;
use nightshade_camera_controller::prelude::{CameraMove, PendingCameraMove};

let camera = spawn_pan_orbit_camera_at(
    commands,
    Vec3::new(0.0, 4.0, 0.0),
    110.0,
    0.8,
    0.5,
    "Main Camera".to_string(),
);

commands.set(
    camera,
    PendingCameraMove {
        kind: CameraMove::Orbit { yaw: 0.2, pitch: 0.0 },
    },
);
```

`spawn_pan_orbit_camera_at` takes the focus, the radius, the yaw, the pitch and the name, `spawn_pan_orbit_camera` takes the defaults, and `spawn_pan_orbit_camera_with_config` takes a whole `PanOrbitCamera` when the bindings or the sensitivity differ. `spawn_third_person_camera` and `spawn_fly_camera_at` spawn the other two shapes. `PendingCameraMove` on a camera is the ask another capability makes, and `apply_camera_requests_system` turns it and takes the component away. A game that wants the camera to avoid geometry writes its own function into `CameraCollisionHook`.

## Taking it from another crate

The crate splits at a `contract` feature, so a crate that only names the types writes `nightshade-camera-controller = { path = "../nightshade-camera-controller", default-features = false, features = ["contract"] }`. The contract carries `PanOrbitCamera` with its bindings and sensitivity, `ThirdPersonCamera`, `CameraCollisionHook`, and the `CameraMove` and `PendingCameraMove` pair, and it enables the camera contract in turn.
