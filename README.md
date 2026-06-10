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
- Full glTF 2.0 support including the KHR extensions: transmission, volume + attenuation, anisotropy, iridescence, dispersion, sheen, clearcoat, diffuse transmission, specular, emissive strength, variants, gpu instancing.
- HDR environments (`.hdr`) and procedural atmospheres (day / night cycle, nebula, space) with prefiltered IBL.
- Skeletal animation, morph targets, and a clip player driven by a frame schedule.
- Data-oriented ECS via [freecs](https://github.com/jessfraz/freecs) with archetype storage and bitflag components.
- Retained-mode UI with Taffy layout, theming, icons, and a built-in test driver.
- Physics ([rapier3d](https://rapier.rs)), audio ([kira](https://github.com/tesselode/kira)), gamepad ([gilrs](https://gitlab.com/gilrs-project/gilrs)), navmesh, particles, decals, picking.
- Scene authoring editor that runs in the browser or as a desktop webview app, with the engine in a web worker, a Leptos interface, gizmos, undo, and Khronos and Polyhaven asset browsers.
- Same code path on native and web. Native uses DX12 / Metal / Vulkan, the browser uses WebGPU.

## Live Demo

[Try the editor in your browser](https://matthewberger.dev/nightshade). Requires a WebGPU-capable browser (Chromium 113+, Firefox 141+, Safari Technology Preview 18+).

<img alt="Nightshade editor running in the browser" src="crates/nightshade/images/live-demo.png" />

## Apps

Example apps in [`apps/`](apps/). Each is buildable with `just run-app <name>`.

<table>
<tr>
<td width="50%" valign="top">
<img src="crates/nightshade/images/apps/winter.png" alt="winter" width="100%" />
<br><b><a href="apps/winter">winter</a></b>
<br><sub>Outdoor snow scene. Atmospheric lighting, large terrain, animated foliage, animated characters.</sub>
</td>
<td width="50%" valign="top">
<img src="crates/nightshade/images/apps/navmesh.png" alt="navmesh" width="100%" />
<br><b><a href="apps/navmesh">navmesh</a></b>
<br><sub>Terrain, grass, and pathfinding. Heightmap-driven grass regions, navmesh baking, AI navigation.</sub>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<img src="crates/nightshade/images/apps/fireworks.png" alt="fireworks" width="100%" />
<br><b><a href="apps/fireworks">fireworks</a></b>
<br><sub>GPU particle system. Bursts, chrysanthemums, rings, glitter, and patterned shape emitters.</sub>
</td>
<td width="50%" valign="top">
<img src="crates/nightshade/images/apps/destructible.png" alt="destructible" width="100%" />
<br><b><a href="apps/destructible">destructible</a></b>
<br><sub>First-person destructible sandbox. Runtime convex mesh slicing, physics rubble, and multiple cutter tools.</sub>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<img src="crates/nightshade/images/apps/lights.png" alt="lights" width="100%" />
<br><b><a href="apps/lights">lights</a></b>
<br><sub>Clustered forward rendering with 1024 dynamic point lights culled per froxel.</sub>
</td>
<td width="50%" valign="top">
<img src="crates/nightshade/images/apps/tower_defense.png" alt="tower_defense" width="100%" />
<br><b><a href="apps/tower_defense">tower_defense</a></b>
<br><sub>Grid-based tower defense with multiple tower types, enemy waves, and pathfinding.</sub>
</td>
</tr>
</table>

## glTF Showcase

Conformance shots of [Khronos glTF Sample Assets](https://github.com/KhronosGroup/glTF-Sample-Assets) rendered in the Nightshade editor. Demonstrates the engine's PBR pipeline and KHR material extensions.

### Transmission & Volume

Refractive glass, volumetric absorption, and chromatic dispersion. Transmissive materials route through weighted-blended OIT with a screen-space scene-color sample for refraction.

<table>
<tr>
<td width="50%" valign="top">
<img src="crates/nightshade/images/gallery/dragon-attenuation.png" alt="Dragon Attenuation" width="100%" />
<br><b>Dragon Attenuation</b>
<br><sub><code>KHR_materials_transmission</code> <code>KHR_materials_volume</code> <code>KHR_materials_ior</code></sub>
<br><sub>Volume attenuation through a thick glass dragon. Beer-Lambert absorption tinting and IOR-driven refraction.</sub>
</td>
<td width="50%" valign="top">
<img src="crates/nightshade/images/gallery/dragon-dispersion.png" alt="Dragon Dispersion" width="100%" />
<br><b>Dragon Dispersion</b>
<br><sub><code>KHR_materials_dispersion</code> <code>KHR_materials_volume</code></sub>
<br><sub>Chromatic dispersion split into per-wavelength refraction directions, producing visible R/G/B fringing on glass edges.</sub>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<img src="crates/nightshade/images/gallery/compare-transmission.png" alt="Compare Transmission" width="100%" />
<br><b>Compare Transmission</b>
<br><sub><code>KHR_materials_transmission</code> <code>Roughness on glass</code></sub>
<br><sub>Smooth and frosted glass bowls side by side. The frosted version exercises roughness applied to refraction.</sub>
</td>
<td width="50%" valign="top">
<img src="crates/nightshade/images/gallery/glass-vase-flowers.png" alt="Glass Vase with Flowers" width="100%" />
<br><b>Glass Vase with Flowers</b>
<br><sub><code>KHR_materials_transmission</code> <code>KHR_materials_volume</code> <code>Multi-mesh transmission</code></sub>
<br><sub>Real-world asset with opaque flora inside transmissive glass. Tests transmission against complex opaque occluders.</sub>
</td>
</tr>
</table>

### Advanced Shading

Anisotropic GGX, thin-film iridescence, sheen, and clearcoat layers stacked on top of the PBR base.

<table>
<tr>
<td width="50%" valign="top">
<img src="crates/nightshade/images/gallery/anisotropy-compare.png" alt="Anisotropy Compare" width="100%" />
<br><b>Anisotropy Compare</b>
<br><sub><code>KHR_materials_anisotropy</code> <code>Anisotropic GGX</code></sub>
<br><sub>Brushed-metal directional reflections driven by anisotropy strength, rotation, and tangent-space direction maps.</sub>
</td>
<td width="50%" valign="top">
<img src="crates/nightshade/images/gallery/iridescence-compare.png" alt="Iridescence Compare" width="100%" />
<br><b>Iridescence Compare</b>
<br><sub><code>KHR_materials_iridescence</code></sub>
<br><sub>Thin-film interference colors that shift with viewing angle. Driven by film thickness and IOR.</sub>
</td>
</tr>
<tr>
<td width="50%" valign="top">
<img src="crates/nightshade/images/gallery/sheen-test.png" alt="Sheen Chair" width="100%" />
<br><b>Sheen Chair</b>
<br><sub><code>KHR_materials_sheen</code> <code>Velvet / cloth</code></sub>
<br><sub>Velvet upholstered chair driven by the sheen extension. Soft Fresnel-based sheen layer stacks on top of the underlying PBR base.</sub>
</td>
<td width="50%" valign="top">
<img src="crates/nightshade/images/gallery/compare-clearcoat.png" alt="Compare Clearcoat" width="100%" />
<br><b>Compare Clearcoat</b>
<br><sub><code>KHR_materials_clearcoat</code> <code>Layered specular</code></sub>
<br><sub>Side-by-side clearcoat variations. A second specular layer over the base material, with its own roughness, models car paint and lacquered finishes.</sub>
</td>
</tr>
</table>

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
        world.resources.render_settings.atmosphere = Atmosphere::Nebula;
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

To bootstrap a new project, use the [nightshade-template](https://github.com/matthewjberger/nightshade-template), which scaffolds the dependencies, justfile, and trunk config.

## Quickstart

Install [just](https://github.com/casey/just), then:

```bash
just init           # installs mise and the pinned editor toolchain (wasm-bindgen, wasm-opt, trunk)
just run            # editor as a desktop app (webview hosting the web build)
just web            # editor in the browser at http://127.0.0.1:8080
just run-app <name> # native app from apps/ (sandbox, game, ...)
cd site && just serve   # full local site with editor + book
```

The editor is a web app either way: `just web` serves it with trunk, and `just run` opens the same bundle in a native webview window. Run `just` with no arguments to list every recipe.

## Documentation

- 📖 [Book](https://matthewjberger.github.io/nightshade/book/) - architecture, systems, and tutorials
- 🦀 [docs.rs](https://docs.rs/nightshade) - generated API reference
- 📚 [USAGE.md](docs/USAGE.md) - engine usage, conventions, gotchas
- 🧪 [UI_TESTING.md](docs/UI_TESTING.md) - retained UI test driver
- 📊 [PROFILING.md](docs/PROFILING.md) - tracing, tracy, chrome-trace
- 🎮 [STEAM.md](docs/STEAM.md) - Steam integration (achievements, stats, friends)
- 🛠️ [STEAM_DECK.md](docs/STEAM_DECK.md) - Steam Deck deployment
- 🚀 [RELEASES.md](docs/RELEASES.md) - release process
- 📜 [LICENSING.md](docs/LICENSING.md) - third-party license notes
- 🤝 [CONTRIBUTING.md](docs/CONTRIBUTING.md) - contributor guidelines (when contributions open)

## Crate Features

Nightshade is split into composable cargo features so apps only pay for what they use. `default = ["engine", "wgpu"]`.

```
Composite:
├── core ─────────────── ECS, math, windowing, time
├── runtime ──────────── core + cosmic-text
├── engine (default) ── runtime + assets + scene_graph + picking + file_dialog + screenshot
└── full ─────────────── engine + wgpu + shell + audio + physics + gamepad + navmesh

Building blocks:
├── assets ───────────── glTF / image / bincode / lz4 asset loading
├── scene_graph ──────── ECS scene round-trip
├── picking ──────────── entity picking
├── file_dialog ──────── native open/save dialogs (rfd)
├── file_watcher ─────── live asset reload (notify)
├── screenshot ───────── capture viewport to PNG
├── shell ────────────── in-game shell commands
└── gizmos ───────────── translate / rotate / scale gizmos

Renderer & subsystems:
├── wgpu ─────────────── GPU rendering (DX12 / Metal / Vulkan / WebGPU)
├── audio ────────────── audio playback (kira)
├── physics ──────────── 3D physics (rapier3d)
├── gamepad ──────────── controller input (gilrs)
├── navmesh ──────────── runtime navmesh queries
├── navmesh-bake ─────── navmesh baking (rerecast)
├── debug_render ─────── line / shape debug overlays
├── steam ────────────── Steam integration (steamworks)
└── windows-app-icon ── embed app icon on Windows

Profiling:
├── tracing ──────────── tracing-subscriber logging
├── tracy ────────────── Tracy profiler integration
└── chrome ───────────── chrome-trace output
```

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
