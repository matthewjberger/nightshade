<p align="center">
  <img src="images/logo.png" alt="nightshade" width="400">
</p>

# Nightshade

A cross-platform data-oriented game engine built with wgpu.

## Getting Started

Use the [nightshade-template](https://github.com/matthewjberger/nightshade-template) to create a new project, or add nightshade to your `Cargo.toml`:

```toml
[dependencies]
nightshade = "0.6.43"
```

### Basic Example

```rust
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(MyGame)?;
    Ok(())
}

struct MyGame;

impl State for MyGame {
    fn title(&self) -> &str {
        "My Game"
    }

    fn initialize(&mut self, world: &mut World) {
        world.resources.user_interface.enabled = true;

        let camera_position = Vec3::new(0.0, 2.0, 10.0);
        let main_camera = spawn_camera(world, camera_position, "Main Camera".to_string());
        world.resources.active_camera = Some(main_camera);
    }

    fn ui(&mut self, _world: &mut World, ui_context: &egui::Context) {
        egui::Window::new("My Game").show(ui_context, |ui| {
            ui.heading("Hello, world!");
        });
    }

    fn run_systems(&mut self, world: &mut World) {
        fly_camera_system(world);
    }
}
```

## Browser Support

WebGPU is supported in:

- All chromium-based browsers (Chrome, Edge, Brave, Vivaldi, etc.)
- Firefox 141+

## License

Nightshade is free, open source and permissively licensed! All code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](https://github.com/matthewjberger/nightshade/blob/main/LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/matthewjberger/nightshade/blob/main/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `nightshade` by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

