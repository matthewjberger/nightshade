<p align="center">
  <img src="images/logo.png" alt="nightshade" width="400">
</p>

# Nightshade

A cross-platform data-oriented game engine built with wgpu.

## Getting Started

Use the [nightshade-template](https://github.com/matthewjberger/nightshade-template) to create a new project, or add nightshade to your `Cargo.toml`:

```toml
[dependencies]
nightshade = "0.1.12"
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

### Custom Window Icon

To use a custom icon for your application window (native) and favicon (WASM):

**Native:** Override the `icon_bytes()` method in your `State` implementation:

```rust
impl State for MyGame {
    fn icon_bytes(&self) -> Option<&'static [u8]> {
        Some(include_bytes!("path/to/your/icon.png"))
    }

    // ... other methods
}
```

**WASM:** Update your `index.html` to reference your icon:

```html
<link data-trunk rel="copy-file" href="path/to/your/icon.png" />
<link rel="icon" type="image/png" href="icon.png" />
```

## Browser Support

WebGPU is supported in:
- All chromium-based browsers (Chrome, Edge, Brave, Vivaldi, etc.)
- Firefox 141+

## License

This project is licensed under the MIT License - see the [LICENSE](https://github.com/matthewjberger/nightshade/blob/main/LICENSE.md) file for details.
