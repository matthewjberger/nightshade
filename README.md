<p align="center">
  <img src="crates/nightshade/images/logo.png" alt="nightshade" width="400">
</p>

<p align="center">
  <a href="https://github.com/matthewjberger/nightshade"><img alt="github" src="https://img.shields.io/badge/github-matthewjberger/nightshade-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20"></a>
  <a href="https://crates.io/crates/nightshade"><img alt="crates.io" src="https://img.shields.io/crates/v/nightshade.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20"></a>
  <a href="https://docs.rs/nightshade"><img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-nightshade-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20"></a>
  <a href="https://matthewjberger.github.io/nightshade/book/"><img alt="book" src="https://img.shields.io/badge/book-nightshade-blue?style=for-the-badge&labelColor=555555&logo=mdbook" height="20"></a>
</p>

`Nightshade` is a data-oriented game engine written in rust 🦀

<img width="2124" height="1440" alt="demo" src="https://github.com/user-attachments/assets/b476d112-c10e-40c3-8dd3-2816153c2a32" />
<img width="2560" height="1440" alt="Screenshot 2025-12-28 121420" src="https://github.com/user-attachments/assets/5ac1e414-a615-4ceb-ac58-0e209cba60f1" />

## Documentation

📖 **[Read the Book](https://matthewjberger.github.io/nightshade/book/)** - Comprehensive guide covering all engine systems, from getting started to advanced features like SDF sculpting and audio-reactive effects.

## Gallery

[View the gallery here](https://matthewberger.dev/nightshade) 🔎
 
<img width="1238" height="1346" alt="image" src="https://github.com/user-attachments/assets/dcfab536-f067-4988-b3f5-d8839444aaea" />

## Getting Started

The fastest way to get started is with the [nightshade-template](https://github.com/matthewjberger/nightshade-template).

## Features

Nightshade uses a modular feature system for fine-grained dependency control.

### Feature Hierarchy

```
nightshade features:
├── core ─────────────── Minimal: windowing, math, ECS, time
├── text ─────────────── 3D text rendering (fontdue)
├── behaviors ────────── Built-in entity behaviors (stateless, WASM-compatible)
├── assets ───────────── Asset loading (gltf, image, bincode, lz4)
├── runtime ──────────── core + text + behaviors
├── engine (default) ─── runtime + assets + scene_graph + terrain + picking + file_dialog + more
├── full ─────────────── engine + egui + audio + physics + scripting + fbx + editor + more
│
│ Add-on features:
├── wgpu ─────────────── GPU rendering (dx12, metal, vulkan, webgpu)
├── egui ─────────────── Immediate mode UI (egui + tiles)
├── audio ────────────── Audio playback (kira)
├── fft ──────────────── FFT audio analysis (rustfft)
├── physics ──────────── 3D physics (rapier3d)
├── gamepad ──────────── Controller input (gilrs)
├── navmesh ──────────── Navigation mesh (rerecast)
├── scripting ────────── Rhai scripting
├── fbx ──────────────── FBX asset format (ufbx)
├── lattice ──────────── Lattice deformation
├── sdf_sculpt ───────── SDF sculpting
├── mosaic ──────────── Multi-pane desktop app framework (egui_tiles)
├── editor ──────────── Scene editor infrastructure (gizmo, undo, inspector, picking)
├── openxr ───────────── VR/AR support
├── steam ────────────── Steam integration
├── mcp ──────────────── AI scene manipulation (MCP server)
├── claude ──────────── Claude Code CLI integration
├── webview ──────────── Embedded webview with IPC
└── plugins ──────────── WASI plugin system
```

### Minimal Build

For a lightweight egui app without asset loading:

```toml
nightshade = { default-features = false, features = ["runtime", "wgpu", "egui"] }
```

## Quickstart

```bash
# native
just run

# wasm (webgpu)
just run-wasm

# serve site
cd site && just serve
```

> All chromium-based browsers like Brave, Vivaldi, Chrome, etc support WebGPU.
> Firefox also [supports WebGPU](https://mozillagfx.wordpress.com/2025/07/15/shipping-webgpu-on-windows-in-firefox-141/) now starting with version `141`.
> Run `just` with no arguments to list all commands

## Prerequisites

* [just](https://github.com/casey/just)
* [trunk](https://trunkrs.dev/) (for web builds)
* [git-cliff](https://git-cliff.org/) (for changelog generation)
* [gh](https://cli.github.com/) (GitHub CLI, for creating releases)

## Platform Guides

### Native (Windows/macOS/Linux)

Build and run natively using cargo:

```bash
just run [project]  # defaults to nightshade-editor
just build          # release build
```

### WebAssembly (WASM)

Build for web browsers with WebGPU support:

```bash
just run-wasm [project]   # defaults to editor
just build-wasm [project]
```

### Steam Deck

See [STEAM_DECK.md](docs/STEAM_DECK.md) for deployment instructions.

### Steam Integration

See [STEAM.md](docs/STEAM.md) for Steam integration (achievements, stats, friends).

### Profiling & Logging

See [PROFILING.md](docs/PROFILING.md) for detailed profiling and logging documentation.

### Plugins

See [PLUGINS.md](docs/PLUGINS.md) for runtime WASI plugin support.

### Scripting

See [SCRIPTING.md](docs/SCRIPTING.md) for Rhai scripting support.

### AI Integration (Native Only)

Nightshade has two features for AI integration: `mcp` for exposing the engine as an MCP server, and `claude` for spawning and communicating with Claude Code CLI as a subprocess.

#### MCP Server (`mcp` feature)

The `mcp` feature starts an HTTP-based [Model Context Protocol](https://modelcontextprotocol.io) server on `http://127.0.0.1:3333/mcp` when the application launches. Any MCP client (Claude Code, Claude Desktop, or custom tooling) can connect and manipulate the running scene in real time.

```toml
nightshade = { features = ["mcp"] }
```

Connect Claude Code:

```bash
claude mcp add --transport http nightshade http://127.0.0.1:3333/mcp
```

**Available tools (50+):**

| Category | Tools |
|----------|-------|
| Entities | `list_entities`, `query_entity`, `spawn_entity`, `despawn_entity`, `rename_entity`, `clear_scene` |
| Transforms | `set_position`, `set_rotation`, `set_scale`, `set_parent`, `set_visibility` |
| Materials | `set_material_color`, `set_emissive`, `set_material`, `set_casts_shadow` |
| Lighting | `spawn_light`, `set_light` |
| Camera | `set_camera` |
| Assets | `load_asset`, `spawn_prefab`, `list_loaded_assets` |
| Environment | `set_atmosphere`, `load_hdr`, `set_graphics` |
| Effects | `spawn_water`, `spawn_particles`, `set_particles`, `spawn_decal` |
| Text | `spawn_hud_text`, `set_hud_text`, `spawn_3d_text` |
| Physics | `add_rigid_body`, `add_collider`, `apply_impulse`, `apply_force`, `set_velocity` |
| Animation | `play_animation`, `stop_animation`, `list_animations` |
| Scripting | `set_script`, `remove_script`, `set_game_state`, `get_game_state` |
| Debug | `add_line`, `add_lines`, `clear_lines`, `get_input`, `get_time` |
| Metadata | `set_entity_tags`, `set_entity_metadata` |
| Batch | `batch` (atomic multi-operation), `run` (concise text commands) |

**Batch operations:** Use `batch` to execute multiple operations atomically in a single frame, reducing round-trip latency.

**Concise commands:** Use `run` for shorthand text commands (e.g. `spawn sun Sphere 0,0,0 scale:2 emissive:5,4,0`).

**State hook:** Applications can intercept MCP commands before the engine processes them by implementing `handle_mcp_command` on the `State` trait. Return `Some(response)` to handle a command yourself, or `None` to let the engine handle it with default behavior.

#### Claude Code CLI (`claude` feature)

The `claude` feature provides types and a worker thread for spawning Claude Code as a subprocess and streaming its output. This lets applications embed an AI assistant panel that can send queries, receive streamed responses (text, thinking, tool use events), and optionally auto-configure MCP so Claude can manipulate the scene.

```toml
nightshade = { features = ["claude"] }
```

Key types: `ClaudeConfig` (system prompt, allowed/disallowed tools, MCP config, custom CLI args), `CliCommand` (StartQuery, Cancel), `CliEvent` (SessionStarted, TextDelta, ThinkingDelta, ToolUseStarted, ToolUseFinished, TurnComplete, Complete, Error).

When both `claude` and `mcp` features are enabled, `McpConfig::Auto` automatically configures Claude Code to connect to the engine's MCP server — no manual `mcp add` step needed.

The editor enables both features by default for AI-assisted development workflows.

## Contributing

Nightshade is currently in its early stages of development. The project is not accepting external contributions at this time as the architecture and core systems are still being established. Once the engine reaches a more mature and stable state, I will open up contributions and provide contributor guidelines.

## License

Nightshade is free, open source and permissively licensed! All code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `nightshade` by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
