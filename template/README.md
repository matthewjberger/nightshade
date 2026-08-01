# Nightshade Template

A starting point for a project built on the [Nightshade](https://github.com/matthewjberger/nightshade)
engine. It lives inside the engine repository and depends on it by relative
path, so it builds against your working copy with no extra clone and no
version to keep in step.

Copy this directory anywhere to start a project. If you move it out of the
repository, repoint every `path = "../crates/..."` dependency in `Cargo.toml`
at wherever your copy of the engine lives.

## Quickstart

```bash
# native
just run

# wasm (webgpu)
just run-wasm

# pc vr (an openxr runtime and a headset)
just run-vr

# steam deck
just build-steamdeck
just deploy-steamdeck        # copies binary to ~/Downloads on deck
just deploy-steamdeck-quick  # copies as 'app' for quick launching
```

> All chromium-based browsers like Brave, Vivaldi, Chrome, etc support WebGPU.
> Firefox also [supports WebGPU](https://mozillagfx.wordpress.com/2025/07/15/shipping-webgpu-on-windows-in-firefox-141/) now starting with version `141`.

## Layout

Each file holds one kind of thing, and every crate in the engine is arranged
the same way:

| File | What goes in it |
|------|-----------------|
| `contract.rs` | The types another crate speaks. No behavior and no engine. |
| `components.rs` | Per-entity data. |
| `resources.rs` | Data shared across entities. |
| `systems/` | Frame behavior, one file per concern. |
| `commands.rs` | Functions that change a resource. |
| `queries.rs` | Functions that only report. |
| `plugin.rs` | Registration, and nothing else. |

Add a `data.rs` when you have plain values or lookup tables to hold. The
engine crates use the same set. `just lint-dag` fails the build if these ever
point at each other in a cycle.

## Prerequisites

* [just](https://github.com/casey/just)
* [trunk](https://trunkrs.dev/) (for web builds)
* [cross](https://github.com/cross-rs/cross) (for Steam Deck builds)
  * Requires Docker (macOS/Linux) or Docker Desktop (Windows)

> Run `just` with no arguments to list all commands

## Features

| Feature | What it adds |
|---------|--------------|
| `contract` | The command and event types. Declares no dependency at all. |
| `plugin` | The components, the systems, and the engine. On by default. |
| `tracing` | Rolling log files at `logs/nightshade.log`. Off by default. |
| `vr` | The OpenXR runner, so the app opens in a headset with the interface on a panel. Off by default. |

A caller that only has to say something to this crate takes the contract on its
own and builds none of the engine:

```toml
template = { path = "../template", default-features = false, features = ["contract"] }
```

The binary declares `required-features = ["plugin"]`, so that build compiles
the contract types alone.

Each capability is a crate of its own under `crates/plugins/`. The template
takes `nightshade-camera` and `nightshade-camera-controller`. Add
`nightshade-physics`, `nightshade-ui`, or `nightshade-steam` to `Cargo.toml`
the same way, put them behind the `plugin`
feature, and compose the matching plugin in `src/main.rs`.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
