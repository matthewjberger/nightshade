# Nightshade Template

A template for creating applications with the [Nightshade](https://github.com/matthewjberger/nightshade) game engine.

## Quickstart

```bash
# native
just run

# wasm (webgpu)
just run-wasm

# openxr (vr headset)
just run-openxr
```

> All chromium-based browsers like Brave, Vivaldi, Chrome, etc support WebGPU.
> Firefox also [supports WebGPU](https://mozillagfx.wordpress.com/2025/07/15/shipping-webgpu-on-windows-in-firefox-141/) now starting with version `141`.

## Prerequisites

* [just](https://github.com/casey/just)
* [trunk](https://trunkrs.dev/) (for web builds)
* [cross](https://github.com/cross-rs/cross) (for Steam Deck builds)
  * Requires Docker (macOS/Linux) or Docker Desktop (Windows)

> Run `just` with no arguments to list all commands

## Optional Features

Enable features with `cargo run --features <feature>`:

| Feature | Description | Docs |
|---------|-------------|------|
| `plugins` | WASI plugin runtime for modding support | [Plugins](https://github.com/matthewjberger/nightshade/blob/main/docs/PLUGINS.md) |
| `scripting` | Rhai scripting for runtime script execution | [Scripting](https://github.com/matthewjberger/nightshade/blob/main/docs/SCRIPTING.md) |
| `tracing` | File logging to `logs/nightshade.log` | [Profiling](https://github.com/matthewjberger/nightshade/blob/main/docs/PROFILING.md) |
| `openxr` | VR headset support | |
| `steam` | Steamworks integration | [Steam](https://github.com/matthewjberger/nightshade/blob/main/docs/STEAM.md) |
| `mcp` | MCP server for AI-assisted scene manipulation | See below |

## MCP Integration (Native Only)

The `mcp` feature exposes an MCP (Model Context Protocol) server that allows AI assistants like Claude to interact with your running application. This enables AI-driven scene manipulation, entity spawning, and real-time control without recompilation.

> **Note:** MCP is only supported on native platforms (Windows, macOS, Linux). It is not available for WASM builds.

### Enabling MCP

Add the feature to your dependencies:

```toml
nightshade = { version = "0.6", features = ["egui", "mcp"] }
```

Or run with the feature flag:

```bash
cargo run --features mcp
```

When enabled, the engine automatically starts an MCP server on `http://127.0.0.1:3333/mcp`.

### Connecting Claude Code

```bash
claude mcp add --transport http nightshade http://127.0.0.1:3333/mcp
```

### Available Tools

| Tool | Description |
|------|-------------|
| `list_entities` | List all named entities in the scene |
| `query_entity` | Get position, rotation, scale of an entity by name |
| `spawn_entity` | Spawn a new entity (Cube, Sphere, Cylinder, Cone, Plane, Torus) |
| `despawn_entity` | Remove an entity by name |
| `set_position` | Set entity position [x, y, z] |
| `set_rotation` | Set entity rotation as euler angles [pitch, yaw, roll] in radians |
| `set_scale` | Set entity scale [x, y, z] |
| `set_material_color` | Set entity material base color [r, g, b, a] |

### Example

With MCP enabled and Claude Code connected, you can say:

> "Spawn a red cube called 'player' at position [0, 1, 0]"

Claude will use the MCP tools to execute:
1. `spawn_entity(name: "player", mesh: "Cube", position: [0, 1, 0])`
2. `set_material_color(name: "player", color: [1.0, 0.0, 0.0, 1.0])`

The editor enables MCP by default for AI-assisted development workflows.

See also: [Steam Deck Deployment](https://github.com/matthewjberger/nightshade/blob/main/docs/STEAM_DECK.md)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
