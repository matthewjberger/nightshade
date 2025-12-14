<p align="center">
  <img src="crates/nightshade/images/logo.png" alt="nightshade" width="400">
</p>

<p align="center">
  <a href="https://github.com/matthewjberger/nightshade"><img alt="github" src="https://img.shields.io/badge/github-matthewjberger/nightshade-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20"></a>
  <a href="https://crates.io/crates/nightshade"><img alt="crates.io" src="https://img.shields.io/crates/v/nightshade.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20"></a>
  <a href="https://docs.rs/nightshade"><img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-nightshade-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20"></a>
</p>

`Nightshade` is a data-oriented game engine written in rust 🦀

 [View the gallery here](https://matthewberger.dev/nightshade) 🔎
 
<img width="1238" height="1346" alt="image" src="https://github.com/user-attachments/assets/dcfab536-f067-4988-b3f5-d8839444aaea" />

## Getting Started

The fastest way to get started is with the [nightshade-template](https://github.com/matthewjberger/nightshade-template).

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

See [STEAM_DECK.md](STEAM_DECK.md) for deployment instructions.

### Steam Integration

See [STEAM.md](STEAM.md) for Steam integration (achievements, stats, friends).

### Profiling & Logging

See [PROFILING.md](PROFILING.md) for detailed profiling and logging documentation.

### Plugins

See [PLUGINS.md](PLUGINS.md) for runtime WASI plugin support.

Quick start:

```toml
# File logging with RUST_LOG filtering
nightshade = { version = "0.6.23", features = ["tracing"] }

# Real-time profiling with Tracy
nightshade = { version = "0.6.23", features = ["tracy"] }

# Offline analysis with Chrome tracing
nightshade = { version = "0.6.23", features = ["chrome"] }
```

## Contributing

Nightshade is currently in its early stages of development. The project is not accepting external contributions at this time as the architecture and core systems are still being established. Once the engine reaches a more mature and stable state, I will open up contributions and provide contributor guidelines.

## License

Nightshade is free, open source and permissively licensed! All code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `nightshade` by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
