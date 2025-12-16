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

| Feature | Description |
|---------|-------------|
| `plugins` | WASI plugin runtime for modding support |
| `tracing` | File logging to `logs/nightshade.log` |
| `openxr` | VR headset support |
| `steam` | Steamworks integration |

## Documentation

- [Profiling & Logging](https://github.com/matthewjberger/nightshade/blob/main/PROFILING.md)
- [Plugin System](https://github.com/matthewjberger/nightshade/blob/main/PLUGINS.md)
- [Steam Integration](https://github.com/matthewjberger/nightshade/blob/main/STEAM.md)

## Plugin Support

This template includes optional WASI plugin support for modding. Plugin support is **disabled by default** and must be explicitly enabled.

### Enabling Plugins

```bash
# Run with plugin support
cargo run --features plugins

# Or via just
just run-plugins
```

When enabled, plugins are loaded from `plugins/plugins/` at runtime.

### Building Plugins

```bash
just build-plugins
```

### Removing Plugin Support Entirely

If you don't need plugin support at all, you can remove it from your project:

1. Remove `plugins/example-plugin` from workspace members in `Cargo.toml`
2. Delete the `plugins/` directory
3. Remove the `plugins` feature from `Cargo.toml`
4. Remove `#[cfg(feature = "plugins")]` blocks from `src/main.rs`

## Steam Deck Deployment

See [Steam Deck documentation](https://github.com/matthewjberger/nightshade/blob/main/STEAM_DECK.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
