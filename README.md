<p align="center">
  <img src="crates/nightshade/images/logo.png" alt="nightshade" width="400">
</p>

<p align="center">
  <a href="https://github.com/matthewjberger/nightshade"><img alt="github" src="https://img.shields.io/badge/github-matthewjberger/nightshade-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20"></a>
  <a href="https://crates.io/crates/nightshade"><img alt="crates.io" src="https://img.shields.io/crates/v/nightshade.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20"></a>
  <a href="https://docs.rs/nightshade"><img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-nightshade-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20"></a>
  <a href="https://matthewjberger.github.io/nightshade/book/"><img alt="book" src="https://img.shields.io/badge/book-nightshade-blue?style=for-the-badge&labelColor=555555&logo=mdbook" height="20"></a>
</p>

`Nightshade` is a data-oriented 3D game engine written in Rust. It targets native (Windows, macOS, Linux) and the web via WebGPU, ships a glTF-first PBR renderer with the full KHR extension set, and comes with a browser-playable scene editor.

<img alt="dragon attenuation rendered in the Nightshade editor" src="crates/nightshade/images/dragon-attenuation.png" />

> Pre-1.0. APIs are still shifting and the project is not yet accepting external contributions. The renderer, editor, and ECS are usable today on Windows, Linux, and modern Chromium / Firefox builds.

## Highlights

- Clustered forward PBR renderer with shadows, IBL, OIT transparency, and post-processing (bloom, SSAO, tone mapping, color grading).
- Full glTF 2.0 support including every relevant KHR extension: transmission, volume + attenuation, anisotropy, iridescence, dispersion, sheen, clearcoat, diffuse transmission, specular, emissive strength, variants, gpu instancing.
- HDR environments (`.hdr`) and procedural atmospheres (day / night cycle, nebula, space) with prefiltered IBL.
- Skeletal animation, morph targets, and a clip player driven by a frame schedule.
- Data-oriented ECS via [freecs](https://github.com/jessfraz/freecs) with archetype storage and bitflag components.
- Retained-mode UI with Taffy layout, theming, icons, and a built-in test driver.
- Physics ([rapier3d](https://rapier.rs)), audio ([kira](https://github.com/tesselode/kira)), gamepad ([gilrs](https://gitlab.com/gilrs-project/gilrs)), navmesh, particles, decals, picking.
- Scene authoring editor that runs natively or in the browser, with multi-viewport docking, gizmos, undo, and a Khronos sample browser.
- Same code path on native and web. Native uses DX12 / Metal / Vulkan, the browser uses WebGPU.

## Hello, Nightshade

```rust
use nightshade::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    launch(SpinCube::default())
}

#[derive(Default)]
struct SpinCube {
    cube: Option<Entity>,
    time: f32,
}

impl State for SpinCube {
    fn initialize(&mut self, world: &mut World) {
        world.resources.graphics.atmosphere = Atmosphere::Nebula;
        capture_procedural_atmosphere_ibl(world, Atmosphere::Nebula, 0.0);
        let camera = spawn_pan_orbit_camera(
            world,
            Vec3::zeros(), 5.0, 0.5, 0.3,
            "Camera".to_string(),
        );
        world.resources.active_camera = Some(camera);
        spawn_sun(world);
        self.cube = Some(spawn_mesh_at(world, "Cube", Vec3::zeros(), Vec3::new(1.0, 1.0, 1.0)));
    }

    fn run_systems(&mut self, world: &mut World) {
        pan_orbit_camera_system(world);
        self.time += world.resources.window.timing.delta_time;
        if let Some(entity) = self.cube
            && let Some(transform) = world.core.get_local_transform_mut(entity)
        {
            transform.rotation = nalgebra_glm::quat_angle_axis(self.time, &Vec3::y_axis());
            mark_local_transform_dirty(world, entity);
        }
    }
}
```

The full annotated version with three objects, materials, and animation lives in the [book's interactive demo](https://matthewjberger.github.io/nightshade/book/interactive-demo.html).

## Live Demo

[Try the editor in your browser](https://matthewberger.dev/nightshade). Requires a WebGPU-capable browser (Chromium 113+, Firefox 141+, Safari Technology Preview 18+).

<img alt="Nightshade editor running in the browser" src="crates/nightshade/images/live-demo.png" />

## Quickstart

Install the tooling:

- [just](https://github.com/casey/just) - task runner
- [trunk](https://trunkrs.dev/) - only required for WASM builds

Then:

```bash
just run            # native (defaults to the editor)
just run-wasm       # browser, WebGPU
cd site && just serve   # full local site with editor + book
```

Run `just` with no arguments to list every recipe.

## Crate Features

Nightshade is split into composable cargo features so apps only pay for what they use.

```
nightshade features:
├── core ─────────────── windowing, math, ECS, time
├── text ─────────────── 3D text rendering (cosmic-text + swash)
├── behaviors ────────── stateless WASM-compatible entity behaviors
├── assets ───────────── asset loading (gltf, image, bincode, lz4)
├── runtime ──────────── core + text + behaviors
├── engine (default) ─── runtime + assets + scene_graph + terrain + picking + file_dialog + more
├── full ─────────────── engine + audio + physics + gamepad + navmesh
│
│ Add-on features:
├── wgpu ─────────────── GPU rendering (DX12, Metal, Vulkan, WebGPU)
├── audio ────────────── audio playback (kira)
├── physics ──────────── 3D physics (rapier3d)
├── gamepad ──────────── controller input (gilrs)
├── navmesh ──────────── navigation mesh (rerecast)
├── grass ────────────── GPU grass rendering
├── terrain ──────────── procedural terrain generation
└── steam ────────────── Steam integration
```

The fastest way to bootstrap a new project is the [nightshade-template](https://github.com/matthewjberger/nightshade-template).

## Documentation

- 📖 [Book](https://matthewjberger.github.io/nightshade/book/) - architecture, systems, and tutorials
- 🦀 [docs.rs](https://docs.rs/nightshade) - generated API reference
- 🛠️ [STEAM_DECK.md](docs/STEAM_DECK.md) - Steam Deck deployment
- 🎮 [STEAM.md](docs/STEAM.md) - Steam integration (achievements, stats, friends)
- 📊 [PROFILING.md](docs/PROFILING.md) - tracing, tracy, chrome-trace

## Maintainer Tools

Only needed for release work, not for using the engine.

- [git-cliff](https://git-cliff.org/) - changelog generation
- [gh](https://cli.github.com/) - GitHub releases

## Contributing

External contributions are not being accepted yet. Architecture and core systems are still settling. Once the engine stabilises, contributor guidelines will land here.

## License

Dual-licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in `nightshade` by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
