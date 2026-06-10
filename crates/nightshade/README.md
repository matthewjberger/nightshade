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
nightshade = "0.34.0"
```

### Hello, Nightshade

```rust
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SpinCube::default())
}

#[derive(Default)]
struct SpinCube {
    cube: Option<Entity>,
    time: f32,
}

impl State for SpinCube {
    fn initialize(&mut self, world: &mut World) {
        world.resources.render_settings.atmosphere = Atmosphere::Nebula;
        capture_procedural_atmosphere_ibl(world, Atmosphere::Nebula, 0.0);
        let camera = spawn_pan_orbit_camera(
            world,
            Vec3::zeros(), 5.0, 0.5, 0.3,
            "Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);
        spawn_sun(world);
        self.cube = Some(spawn_mesh_at(world, "Cube", Vec3::zeros(), Vec3::new(1.0, 1.0, 1.0)));
    }

    fn run_systems(&mut self, world: &mut World) {
        pan_orbit_camera_system(world);
        self.time += world.resources.window.timing.delta_time;
        if let Some(entity) = self.cube
            && let Some(transform) = world.core.get_local_transform_mut(entity)
        {
            transform.rotation = nalgebra_glm::quat_angle_axis(self.time, &Vec3::y_axis());
            mark_local_transform_dirty(world, entity);
        }
    }
}
```

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
