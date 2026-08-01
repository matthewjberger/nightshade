# nightshade-window

The winit loop and the two traits a backend implements to sit under it. The crate names no GPU API: it opens the window, pumps the events into the platform resources, drives the frame, and hands the drawing to whatever `Renderer` it was given. It also carries the frame pacing, the file drop handling and the plugin group every program composes.

## Composing it

```rust
App::new()
    .add_plugin(nightshade_vulkan::RenderPlugin::default())
    .add_plugin(nightshade_window::WindowPlugin::default())
    .add_plugin(nightshade_cli::CliPlugin::<Arguments>::default())
    .add_plugin(nightshade_log::LogPlugin::default())
    .add_plugin(nightshade_camera::CameraPlugin)
    .run()
```

`WindowPlugin` composes `PlatformPlugin`, `ScenePlugin` and `GraphicsPlugin`, inserts `PendingHost`, writes the title and the initial size onto the platform's `Window` resource, and registers `process_dropped_files_system` on `Stage::First`. `DefaultPlugins<A, R>` is that plugin beside the command line, the log, the backend and the camera in one group, and each backend hands out an alias of it so a program that wants one member different sets that member with `.set(plugin)`.

## Using it

```rust
pub fn run_windowed(app: App) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = app;
    let log_guards = app.take_log_guards();
    let (world, host, hooks) = crate::commands::into_parts(app);
    let result =
        nightshade_window::run::launch_with_world(Box::new(host), renderer_factory(hooks), world);
    drop(log_guards);
    result
}
```

A backend sets this as the app's runner. `launch_with_world` takes a boxed `Host`, a `RendererFactory` that builds the renderer once the window exists, and the world to run, while `launch` is the same call over a fresh tracking world. `Host` is the three backend-free calls, `initialize`, `run_systems` and `window_event`. `Renderer` is the seven the frame needs, `configure`, `resize`, `release_surface`, `recreate_surface`, `update`, `prepare` and `render`, each reporting a `RenderError` by string, and neither trait names a GPU type. `throttle_to_frame_rate_limit` and the macOS refresh rate calls in `pacing` hold the frame to what `FramePacing` asks for.

## Taking it from another crate

The crate does not split at a `contract` feature. A backend takes it whole for the two traits and the loop, and enables its `default-plugins` feature to get the `DefaultPlugins` group it hands out under its own name.
