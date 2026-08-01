# nightshade-app

nightshade-app is the composition every program repeats before it adds anything of its own. One call builds the `App`, composes the backend's `DefaultPlugins`, and writes the title, the capture frame, the window size and the frame pacing from a single `Opening`. A program says what it is and then adds only the plugins that are its own.

## Composing it

The crate carries no plugin. It hands out the `compose` function that builds the `App` the other plugins are added to.

```rust
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<nightshade_app::NoArguments>(
        nightshade_app::Opening::titled("Ray Traced Audio"),
    );
    app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_audio::AudioPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .run()
}
```

`compose` adds the backend group its `wgpu` or `vulkan` feature selects, setting `CliPlugin`, `LogPlugin`, `WindowPlugin` and `RenderPlugin` from the `Opening`. It writes the title onto the `Window` resource and registers nothing else, so a program that wants the escape key to close the window registers `escape_key_exit_system` on `Stage::FrameUpdate` itself, and one with a menu reads the key in its own system.

## Using it

```rust
use nightshade_app::{Opening, compose};

let mut app = compose::<SdfArguments>(
    Opening::titled("Signed Distance Fields")
        .settling_at(300)
        .sized((1280, 720))
        .limited_to(144.0),
);
```

`Opening::titled` names the window and takes the defaults for the rest. `settling_at` raises the frame a scheduled capture is taken on, for a scene that takes longer to arrive. `sized` opens the window at a fixed size, `limited_to` and `pacing` set the frame rate limit, `without_vsync` turns vsync off. The type parameter of `compose` is the app's own `clap::Args` struct, which `CliPlugin` parses and inserts as a resource, and `NoArguments` is what an app with no flags of its own passes.

## Taking it from another crate

Nothing asks this crate for types. It sits above the backends and composes them, so a plugin crate never takes it and only a program does.
