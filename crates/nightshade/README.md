<p align="center">
  <img src="images/logo.png" alt="nightshade" width="400">
</p>

# Nightshade

A data-oriented 3D game engine written in Rust. Targets native (Windows, macOS, Linux) and the web via WebGPU. Ships a glTF-first PBR renderer with the full KHR extension set, a freecs-backed ECS, a retained-mode UI, and a browser-playable scene editor.

See the [project README](https://github.com/matthewjberger/nightshade) for screenshots, the live demo, and the project status.

## Getting Started

Use the [nightshade-template](https://github.com/matthewjberger/nightshade-template) to scaffold a new project, or add nightshade to your `Cargo.toml`:

```toml
[dependencies]
nightshade = "0.51.0"
```

### Hello, Nightshade

```rust
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SpinCubePlugin)
        .run()
}

#[derive(Default)]
struct SpinCube {
    cube: Option<Entity>,
    time: f32,
}

struct SpinCubePlugin;

impl Plugin for SpinCubePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpinCube::default());
        app.add_game_startup_system(initialize);
        app.add_game_system(Stage::Update, spin);
    }
}

fn initialize(game: &mut SpinCube, world: &mut World) {
    world.resources.render_settings.atmosphere = Atmosphere::Nebula;
    let camera = spawn_pan_orbit_camera(
        world,
        Vec3::zeros(), 5.0, 0.5, 0.3,
        "Camera".to_string(),
    );
    world.resources.active_camera = Some(camera);
    spawn_sun(world);
    game.cube = Some(spawn_mesh_at(world, "Cube", Vec3::zeros(), Vec3::new(1.0, 1.0, 1.0)));
}

fn spin(game: &mut SpinCube, world: &mut World) {
    pan_orbit_camera_system(world);
    game.time += world.resources.window.timing.delta_time;
    if let Some(entity) = game.cube
        && let Some(transform) = world.get_mut::<LocalTransform>(entity)
    {
        transform.rotation = nalgebra_glm::quat_angle_axis(game.time, &Vec3::y_axis());
    }
}
```

## Architecture

The engine sits on top of a standalone renderer:

- `nightshade` owns the ECS, the frame schedule, physics, animation, audio, input, UI, asset loading, and the render driver that snapshots the scene into the renderer's per-frame inputs.
- [`nightshade-renderer`](https://crates.io/crates/nightshade-renderer) is a standalone GPU-driven wgpu renderer. It consumes one owned `RenderInputs` value per frame and never sees the ECS world. Its frame graph (pass scheduling, transient resource aliasing, store-op optimization) is usable on its own through the `rendergraph` feature.

Each layer only knows the one below it.

## Features

`default = ["engine", "wgpu"]`. Opt in to more with `cargo run --features <name>`.

| Feature | What it adds |
|---------|--------------|
| `engine` | Default. Runtime + asset loading + scene graph + picking + file dialogs + screenshot |
| `wgpu` | Default. GPU rendering (DX12 / Metal / Vulkan / WebGPU) |
| `full` | `engine` + `wgpu` + `audio` + `physics` + `gamepad` + `navmesh` + `shell` |
| `audio` | Audio playback (kira) |
| `physics` | 3D physics (rapier3d) |
| `gamepad` | Controller input (gilrs) |
| `navmesh` | Runtime navmesh queries |
| `navmesh-bake` | Navmesh baking (rerecast) |
| `shell` | In-game shell commands |
| `gizmos` | Translate / rotate / scale gizmos |
| `debug_render` | Line / shape debug overlays |
| `file_watcher` | Live asset reload (notify) |
| `screenshot` | Capture viewport to PNG |
| `steam` | Steam integration (steamworks) |
| `tracing` | tracing-subscriber logging |
| `tracy` | Tracy profiler integration |
| `chrome` | chrome-trace profiler output |
| `windows-app-icon` | Embed app icon on Windows |

## Browser Support

WebGPU is supported in:

- All chromium-based browsers (Chrome, Edge, Brave, Vivaldi, etc.)
- Firefox 141+
- Safari Technology Preview 18+

## License

Dual-licensed under either of:

- MIT License ([LICENSE-MIT](https://github.com/matthewjberger/nightshade/blob/main/LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/matthewjberger/nightshade/blob/main/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `nightshade` by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
