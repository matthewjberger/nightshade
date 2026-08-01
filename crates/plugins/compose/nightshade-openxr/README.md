# nightshade-openxr

`nightshade-openxr` opens a program on a headset. It stands up the OpenXR session, drives the main loop itself, spawns a rig of entities for the origin, the eyes, the head, the two hands and the pointer, and publishes the head and controller state each frame as resources a game reads. It also anchors the retained UI on a quad in front of the viewer and draws a cursor where the pointer meets it.

## Composing it

```rust
App::new()
    .add_plugin(nightshade_window::WindowPlugin::default())
    .add_plugin(nightshade_log::LogPlugin::default())
    .add_plugin(nightshade_wgpu::RenderPlugin::default())
    .add_plugin(nightshade_camera::CameraPlugin)
    .add_plugin(OpenXrPlugin::default())
    .run()
```

`OpenXrPlugin` inserts `XrSettings`, `XrFrame`, `XrInput`, `XrUiPanel` and `XrUiPointer`, spawns the rig on `Stage::Startup`, reads the poses and the UI pointer on `Stage::First`, paints the cursor on `Stage::Last`, and takes over the runner.

## Using it

```rust
fn drive(
    xr_frame: Res<XrFrame>,
    xr_input: Res<XrInput>,
    mut panel: ResMut<XrUiPanel>,
) {
    if xr_input.a_pressed() {
        panel.enabled = !panel.enabled;
    }
    if xr_frame.tracking_valid {
        let stick = xr_input.left.thumbstick;
        panel.distance = (panel.distance + stick.y * 0.01).max(0.3);
    }
}
```

`XrFrame` carries the head pose, the per-eye views and whether tracking is valid. `XrInput` carries a thumbstick, a trigger, a grip and an aim pose for each hand, with `a_pressed` and `b_pressed` reporting the edge. `XrUiPanel` says where the UI quad hangs, either on the head or at a fixed `XrUiAnchor::Stage` pose, and `XrUiPointer` reports where the ray lands on it. The rig arrives as an `XrRig` resource, so a program parents a model to `rig.hand(XrHand::Left)` or moves `rig.origin` to walk the player around.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-openxr = { path = "../nightshade-openxr", default-features = false, features = ["contract"] }
```

The contract carries the rig, the frame, the input state, the panel and pointer settings, and the eye and hand enumerations, with no session and no runner behind them.
