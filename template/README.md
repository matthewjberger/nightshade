# Nightshade Template

A template for creating applications with the [Nightshade](https://github.com/matthewjberger/nightshade) game engine.

## Quickstart

```bash
# native
just run

# wasm (webgpu)
just run-wasm

# openxr (vr headset)
just run-xr
```

> All chromium-based browsers like Brave, Vivaldi, Chrome, etc support WebGPU.
> Firefox also [supports WebGPU](https://mozillagfx.wordpress.com/2025/07/15/shipping-webgpu-on-windows-in-firefox-141/) now starting with version `141`.

## Prerequisites

* [just](https://github.com/casey/just)
* [trunk](https://trunkrs.dev/) (for web builds)
* [cross](https://github.com/cross-rs/cross) (for Steam Deck builds)
  * Requires Docker (macOS/Linux) or Docker Desktop (Windows)

> Run `just` with no arguments to list all commands

## Steam Deck Deployment

Deploy to Steam Deck using `just deploy-steamdeck`. First-time setup on Steam Deck (must be in desktop mode):

1. Set password for `deck` user: `passwd`
2. Enable SSH: `sudo systemctl enable sshd && sudo systemctl start sshd`
3. Deploy the binary: `just deploy-steamdeck`
4. Add `~/Downloads/nightshade-template` as a non-steam game in Steam
5. Launch from Big Picture mode or Game mode after initial setup
6. Future deploys must be done from desktop mode, but the last deployed binary will run in game mode

## Profiling & Logging

The `tracing` feature is enabled by default, providing:
- Rolling daily log files in `logs/`
- `RUST_LOG` environment variable filtering

```bash
RUST_LOG=debug just run
RUST_LOG=info,my_game=trace just run
```

For real-time profiling with Tracy or offline Chrome tracing, see the [Nightshade profiling documentation](https://github.com/matthewjberger/nightshade/blob/main/PROFILING.md).

## Plugin Support

This template includes WASI plugin support for modding by default. Plugins are loaded from `plugins/plugins/` at runtime.

> **Note:** Plugins are only supported on native builds (Windows, macOS, Linux). WASM/web builds do not support plugins.

### Building the Example Plugin

```bash
# Install the wasm32-wasip1 target
rustup target add wasm32-wasip1

# Build the example plugin
cargo build -p example-plugin --target wasm32-wasip1 --release

# Copy to plugins directory
cp target/wasm32-wasip1/release/example_plugin.wasm plugins/plugins/
```

### Creating Your Own Plugins

1. Create a new crate in `plugins/` with `crate-type = ["cdylib"]`
2. Use `nightshade::plugin` types for the API:
   ```rust
   use nightshade::plugin::{log, spawn_cube, drain_engine_events, EngineEvent};

   #[unsafe(no_mangle)]
   pub extern "C" fn on_init() {
       log("Plugin initialized!");
       spawn_cube(0.0, 1.0, -5.0);
   }

   #[unsafe(no_mangle)]
   pub extern "C" fn on_frame() {
       for event in drain_engine_events() {
           // Handle events
       }
   }
   ```

See the [full plugins documentation](https://github.com/matthewjberger/nightshade/blob/main/PLUGINS.md) for the complete API.

### Editor Plugins

Editor plugins extend the Nightshade Editor with custom inspectors, menu items, and functionality. They use the `nightshade-editor-api` crate.

```bash
# Build the editor plugin
cargo build -p editor-plugin --target wasm32-wasip1 --release

# Copy to plugins directory
cp target/wasm32-wasip1/release/editor_plugin.wasm plugins/plugins/
```

Example editor plugin:

```rust
use nightshade_editor_api::{
    drain_events, drain_inspector_requests, log, register_inspector,
    respond_inspector_ui, send_mod_info, EditorEvent, InspectorRequest,
    ModInfo, UiElement,
};

#[unsafe(no_mangle)]
pub extern "C" fn on_init() {
    send_mod_info(&ModInfo {
        id: "my-plugin".to_string(),
        name: "My Plugin".to_string(),
        version: "0.1.0".to_string(),
    });
    register_inspector("My Data", "my_data");
}

#[unsafe(no_mangle)]
pub extern "C" fn on_frame() {
    for event in drain_events() {
        // Handle editor events
    }
    for request in drain_inspector_requests() {
        // Render custom inspector UI
    }
}
```

### Opting Out of Plugin Support

If you don't need plugin support, you can remove it to reduce binary size and dependencies:

1. **Remove the `plugins` default feature** in `Cargo.toml`:
   ```toml
   [features]
   default = []  # Remove "plugins" from default
   ```

2. **Remove the workspace members** in `Cargo.toml`:
   ```toml
   [workspace]
   members = []  # Remove "plugins/example-plugin" and "plugins/editor-plugin"
   ```

3. **Delete the plugins directory**:
   ```bash
   rm -rf plugins/
   ```

4. **Remove plugin code from `src/main.rs`** - delete or comment out:
   - The `#[cfg(feature = "plugins")] mod plugin_runtime;` line
   - The `plugin_runtime` field from the `Template` struct
   - All `#[cfg(feature = "plugins")]` blocks

5. **Delete `src/plugin_runtime.rs`**

## Steam Integration

To enable Steamworks integration (achievements, stats, friends):

1. Enable the `steam` feature as default in `Cargo.toml`:
   ```toml
   [features]
   default = ["plugins", "steam"]
   steam = ["nightshade/steam"]
   ```

2. Create `steam_appid.txt` in project root (use `480` for testing, replace with your App ID for release):
   ```
   480
   ```

3. Initialize Steam in your game:
   ```rust
   fn initialize(&mut self, world: &mut World) {
       world.resources.steam.initialize().ok();
   }
   ```

4. Include Steam redistributable DLLs with your release build:
   - Windows: `steam_api64.dll`
   - Linux: `libsteam_api.so`
   - macOS: `libsteam_api.dylib`

See the [full Steam documentation](https://github.com/matthewjberger/nightshade/blob/main/STEAM.md) for API reference and using your own App ID.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
