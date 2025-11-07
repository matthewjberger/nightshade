# Nightshade

A cross-platform data-oriented game engine built with wgpu.

## Getting Started

Add nightshade to your `Cargo.toml`:

```toml
[dependencies]
nightshade = "0.1.2"
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

This project is licensed under the MIT License - see the [LICENSE](https://github.com/matthewjberger/nightshade/blob/main/LICENSE.md) file for details.
