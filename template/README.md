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

See also: [Steam Deck Deployment](https://github.com/matthewjberger/nightshade/blob/main/docs/STEAM_DECK.md)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
