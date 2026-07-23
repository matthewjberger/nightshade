---
description: Orient a new session on the nightshade engine. Architecture, where every detail lives, and how to work here without blowing context.
---

# Orient: Nightshade Engine

You are starting work in the nightshade engine repo. This is your orientation. Read it in full, build the mental model below, then load only the specific docs a task actually needs. Do NOT read all of `docs/` up front. It is large (`USAGE.md` alone is ~3000 lines) and will blow your context. This file is the map. Load the territory only as a task needs it.

## What this is

A cross-platform data-oriented game engine (wgpu, nightshade_ecs ECS, nalgebra_glm) shipped as two layers.

- **`nightshade-api`** (`crates/nightshade-api`) is the high-level facade: roughly 300 free functions over `World` and `Entity`, plain data, no trait to implement. `open()`, `run()`, and `run!` boot a working scene (sky, sun, shadows, grid, orbit camera, prototype textures). Start here for a game.
- **`nightshade`** (`crates/nightshade`) is the engine itself: the plugin-composed `App` builder, the ECS, the render graph, every subsystem. Drop to it one call at a time when the facade stops covering what you need, via `use nightshade_api::nightshade::prelude::*;`.

Every facade function bottoms out in a real engine call and hands you the real `World`, so nothing is hidden and there is no migration cost.

## Step 1: pick your layer before writing anything

- Building or extending a game, scene, or demo: use the facade. Read `docs/SPEC_RUST_API.md` (the full signature catalog, written to be read once) and the "Start Here" section of `docs/USAGE.md`. Working references: `apps/coin_dash`, `apps/coin_rush`, `template_api/`, and `crates/nightshade-api/examples`.
- Building or extending the engine, a custom render pass, a new component or system, or an engine-layer app: use the engine layer. Read the relevant section of `docs/USAGE.md` and the canonical engine-layer app `apps/sandbox` before writing.

If unsure, default to the facade and only descend when a specific need forces it.

## The mental model (hold this, do not re-derive it)

- **The engine boundary is two streams.** Commands flow in, events flow out. Anything outside core (native logic, rhai, WASM, remote, a binding) sits on those two data streams and never gets `&mut World`. Two invariants keep it bounded and deterministic: commands emit events and never more commands, and participants run once per cycle in fixed order. Command replies are immediate (a spawn hands back a real `Entity` the same tick), but the events a command emits land one cycle later. Detail in `docs/COMMANDS_AND_EVENTS.md`.
- **ECS is nightshade_ecs dynamic worlds.** The engine `World` is a thin `{ ecs: DynEcs }` wrapper — there is no `resources` field. The group has three engine member worlds, declared in `src/ecs/world.rs` with `nightshade_ecs::dynamic_worlds!`: `world.ecs.worlds[CORE]` (the scene), `world.ecs.worlds[UI]` (retained UI), and `world.ecs.worlds[RENDER]` (the renderer's scene snapshot, kept current by the delta sync). Apps add a fourth member world, `GAME` (index 3), for their own components. Resources are singletons living in the ECS resource map, not fields on a struct: read them as `Res<T>` / `ResMut<T>` system parameters, or `world.res::<T>()` / `world.res_mut::<T>()` inside a `&mut World` system. Components register eagerly in `src/ecs/world.rs` (`nightshade_ecs::dynamic_schema!`), which emits the bitflag masks (`CAMERA | LOCAL_TRANSFORM`); registration order is the schema. Point access is typed and group-routed: `world.get::<Camera>(entity)`, `world.get_mut::<T>`, `world.set(entity, value)`, `world.has::<T>` resolve the owning member world per component type. The default single-entity spawn is a bundle, `world.spawn_with((component, ...))`, deriving the archetype from the component types; mask spawn (`spawn_entities`) covers batches and all-default rows. The default query is a typed tuple: `query_ref::<(&A, &B)>().iter()` on `&world` for reads, `query::<(&mut A, &B)>().for_each(...)` for writes (`&mut` elements stamp change ticks), with `.with::<T>()` / `.without::<T>()` / mask variants for filters; systems also take `Query<...>`, `Res<T>`, `ResMut<T>`, `EventReader<T>`, and `ParamSet` parameters. For structural mutation during iteration, gather `(entity, payload)` into a `Vec`, then apply in a second pass.
- **Most engine systems run automatically** in the frame stages (`Stage::FrameStart`, `Stage::FrameUpdate`, `Stage::FramePostUpdate`, `Stage::Render`, `Stage::Last`) that follow the game stages each frame in one `Stages` run: the core systems land first and each composed plugin registers its own with `app.add_system(Stage::..., system)`; within a stage, systems run in registration order. Do not call these manually. Camera controllers and a few others (cutscenes) are opt-in from the game plugin's update systems. Never hand-run an engine system from app code. Fix the data dependency instead.
- **App member-world pattern.** A game declares its components with `nightshade_ecs::dynamic_schema!` and registers them as the app member world with `world.ecs.add_world_at(GAME, ...)` (`GAME = 3`, the first slot after CORE/UI/RENDER), sharing entity identity with the scene entities: game state is set directly on the engine entity, and one recursive despawn tears down both. App-wide state is a plain resources struct the plugin inserts with `app.insert_resource(...)` into the same ECS resource map as the engine's, read back as `Res<T>` / `ResMut<T>` params (not owned by a `State` type, not reached through `game_scope`). `apps/mini_dungeon` and `apps/tower_defense` are the live patterns; `template/` is the skeleton.
- **Leptos web apps are a plugin pair.** In the worker, `nightshade_api::offscreen::WorkerHostPlugin` (behind the `offscreen` feature) claims the main loop through `App::set_runner` and owns the canvas, the render loop, and the page conversation, generic over the app's `WorkerProtocol` message pair with binary attachments and a `WorkerOutbox` reply resource. On the page, `nightshade_api::web` (behind `leptos`, engine-free) provides `use_engine`/`use_engine_typed`, `EngineViewport`, the widget library, and the file plumbing. The editor, playground, and self_assembling compose it; the templates are the skeletons.
- **Rendering is a standalone crate** (`crates/nightshade-renderer`, re-exported as `nightshade::render`). It consumes one owned `RenderInputs` per frame and never sees the ECS world. A delta scene sync keeps its input snapshot (the `RENDER` member world) current from the structural change log and change ticks. Its frame graph (`RenderGraph<RenderInputs>`) has passes declare slot names while the graph resolves dependencies, store and clear ops, transient aliasing, and dead-pass culling. Every camera renders to its own offscreen viewport texture and the swapchain is composed from those. The draw path is GPU-driven: the GPU culls, classifies, bins, and builds indirect commands, while the CPU does delta uploads plus one fixed indirect draw per pipeline class. Detail in `docs/RENDERER.md` and `docs/GPU_DRIVEN_SCENE.md`.
- **Windowing and input are a standalone crate** (`crates/nightshade-platform`, re-exported so the old `crate::ecs::input::…` and `crate::ecs::window::…` paths still resolve). Built on `nightshade-ecs` and sitting beside the renderer, it holds the `Input` and `Window` resources, the monitor registry, the winit event decoder, the gilrs poll, and the `run_event_loop` driver. The engine keeps the frame orchestration: its `WindowContext` implements winit's `ApplicationHandler` and is what `run_event_loop` drives, and thin engine wrappers add the event-bus publish and app-event forwarding on top of the platform decoder. The engine `Viewport` resource holds the viewport and camera-tile render state that used to live on `Window`.
- **The retained UI is a standalone crate** (`crates/nightshade-ui`, re-exported module by module as `crate::ecs::ui`, which is the engine's supported surface: the widget vocabulary, the state the host drains, and the entry points, with the UI's internal pipeline modules left out because `UiPlugin` runs them). Built on `nightshade-ecs`, `nightshade-text`, `nightshade-platform`, and one module of `nightshade-renderer`, its systems take `&mut DynEcs` (not the engine `World`) and query `world.worlds[UI]`. It owns the UI schema (`register_ui_components`, which the engine composes with `add_world_at(UI, …)`) and takes its member-world slots from the host through the `UiWorldIndices` resource rather than hardcoding them, and exposes `default_ui_systems()`, which the engine's `RetainedUiSchedule` runs each frame. Draw data goes to `nightshade-renderer`, and window, input, clipboard, browser, and the AccessKit adapter come from `nightshade-platform`, so the UI names no windowing type. Shared text (`TextState`, `TextProperties`) lives in the `nightshade-text` leaf, `EasingFunction` in the renderer, and the `Time` clock in `nightshade-ecs`, so both the UI and the engine reach them.

## Where every detail lives (read the one that matches your task, never all)

| Doc | Read it when you need |
| --- | --- |
| `docs/SPEC_RUST_API.md` | The facade signature catalog. First stop for any game work. |
| `docs/USAGE.md` | Engine-layer usage for every subsystem (meshes, cameras, lighting, materials, physics, audio, animation, UI, scenes). Grep or section-read it, do not read the whole file. |
| `docs/ENGINE_FEATURES.md` | Index of every shipped mechanic and which app exercises it. Best "does the engine already do X, and where is the example". |
| `docs/RENDERER.md` | Render graph architecture, every built-in pass, per-frame pipeline, camera viewports, custom passes. |
| `docs/GPU_DRIVEN_SCENE.md` | The GPU scene, culling, and indirect-draw architecture, and what stays on the CPU by design. |
| `docs/COMMANDS_AND_EVENTS.md` | The command and event boundary, the two invariants, the timing model. |
| `docs/COMMAND_API.md` | The serializable `Command` enum, `Ref`, batch wiring, how bindings drive the engine. |
| `docs/SCRIPTING.md`, `docs/SPEC_RHAI_MCP.md` | The rhai scripting model and the editor MCP tool surface. |
| `docs/EDITOR_AGENT.md` | Driving the live editor over MCP (the bridge architecture and tool catalog). |
| `docs/UI_TESTING.md` | `UiTestDriver` frame timing and assertions for retained UI. |
| `docs/PROFILING.md` | The Tracy, chrome, and tracing workflow for frame-time investigation. |
| `docs/STEAM.md`, `docs/STEAM_DECK.md`, `docs/RELEASES.md`, `docs/LICENSING.md` | Platform integration, deploy, release, and license tooling. |
| `docs/C_BINDINGS.md`, `docs/GO_BINDINGS.md`, `docs/PYTHON_BINDINGS.md` | FFI bindings over the command surface. |
| `docs/book/` (mdBook), `docs/by-example/` | Long-form tutorials and recipe-per-page. Reference, not required reading. |

## Reference code (copy proven patterns, do not invent new architecture)

- `apps/sandbox`: the canonical engine-layer 3D game. A `SandboxPlugin` in `plugin.rs` registers startup and update systems with `add_system`/`add_systems` onto free functions in `systems/<domain>` (systems take `Res<T>` / `ResMut<T>` / `EventReader<T>` params plus `&mut World`), an app member world for game components (`ecs.rs`), screens driven by app states (`insert_state` / `on_enter` / `while_in` / `next_state`) with the per-screen config in `systems/lifecycle.rs`, an app resources struct (`SandboxResources`) inserted with `app.insert_resource`, `theme.rs` for UI constants. Read it before writing a new engine-layer app.
- `apps/coin_dash`, `apps/coin_rush`: idiomatic facade games. `run!(setup, ...systems)`, state in a plain struct holding `Entity` handles.
- `template/` (engine skeleton), `template_api/` (facade skeleton): the same architecture stripped to a starting point.
- Per-feature apps: `fireworks` (particles), `cloth`, `movement` (physics queries), `gallery` and `ui_test_harness` (retained UI), `navmesh`, `lights` (clustered forward, 1024 point lights), `ssr`, `decals`, `alpha_blending` (custom render-graph configs), `water`, `vfx`, `cutscene`, `editor` (the densest app). `docs/ENGINE_FEATURES.md` maps features to apps.

## How to work here (the method that protects context)

1. **Delegate broad searches.** For "where is X", "who calls Y", or any sweep across many files, spawn an `Explore` agent and keep only its conclusion. Do not read a pile of files into your own context.
2. **Read the one relevant doc section**, not the whole doc. `USAGE.md` and `SPEC_RUST_API.md` are large. Grep for the subsystem or read a bounded range.
3. **Read whole files before editing them.** Otherwise you duplicate code or misread the architecture.
4. **Reuse existing patterns.** Find the nearest working app or example and follow it. Respect the `systems/<name>.rs` architecture: behavior goes in a system module, not lumped into `state.rs`.
5. **When a search drives a removal or refactor**, double-check for other consumers across all apps before declaring it done.

## Hard rules (non-negotiable, full set in the root `CLAUDE.md` and the user's global config)

- Rust: no comments ever (only `///` or `//!` rustdoc when it earns its place), no debug logging, no abbreviations (`index`, not `i`), no `unsafe` unless told, no `mod.rs` (2024 edition), no `#[allow(...)]` to silence a lint (fix the cause), no `Rc<RefCell<T>>`.
- `nalgebra_glm` only, never `glam`. Use `Vec2::component_mul()` for element-wise multiply. WGSL uniform structs need 16-byte alignment, so pack with `vec4`, not `vec3`.
- Data-oriented, not OOP. No "Manager" suffixes, no one-line forwarding wrappers, no gratuitous `pub`. The builder pattern is the lone allowed OOP-ish exception.
- When you replace a function, delete the old one and its now-orphaned helpers and imports in the same change. Never leave both.
- Never default a new capability off. Build it as THE path, not a flag that ships false.
- Transform writes through `world.set(entity, LocalTransform { .. })` or `world.get_mut::<LocalTransform>(entity)` propagate automatically via ECS change ticks. `nightshade_ecs::Entity` is the entity type everywhere.
- Windows host: use PowerShell-compatible commands and the dedicated file tools, not the Bash tool, for path operations.

## Build, run, and the commit gate

Use the `justfile`, never raw `cargo run`.

- `just run <project>` runs a native app (blank arg opens the editor). Use `just run-app <project> -- <args>` to pass args.
- `just run-example <name>` runs a facade example from `crates/nightshade-api/examples`. Use `just list-examples` to enumerate them.
- `just run-wasm <project>` does a browser build via Trunk.
- Gate before any commit: `just format`, then `just lint` (clippy across the workspace with warnings as errors, which also runs the wasm lint), then `just test`. Confirm green.
- In PowerShell 5.1 do not pipe `2>&1` from cargo or just. It false-fails `$LASTEXITCODE`.
- Git only when explicitly told. Conventional Commits (`type(scope): description`). No em-dashes, semicolons, `@`, or attribution or Co-Authored-By lines in messages. Never push without the user verifying.

## Adding a component or system

- **App-level game data** does not touch engine source. Declare the components with `nightshade_ecs::dynamic_schema!` and register them as the app member world (`world.ecs.add_world_at(GAME, register_<app>_components())`), then set game state directly on engine entities. See `apps/sandbox/src/ecs.rs`. App-wide singletons go in a plain struct inserted with `app.insert_resource(...)` and read as `Res<T>` / `ResMut<T>`.
- **Plugin-owned subsystems** (audio, gamepad, physics, steam, egui, log, the UI composition) live self-contained under `src/plugins/<name>`: systems, components, resources, and the plugin struct together, crate-shaped on purpose but feature-gated into this crate. Composing the plugin is what activates the capability.
- **Engine-level component.** Create the module at `src/ecs/<name>.rs` plus `src/ecs/<name>/components.rs`, with systems in `src/ecs/<name>/systems.rs`. Register it in `src/ecs/ecs.rs` (`pub mod <name>;`) and in `src/ecs/world.rs`: append one `field: Type => CONST,` line to the matching `nightshade_ecs::dynamic_schema!` block (append only, registration order is the snapshot schema) and add the type to the `pub mod components` re-exports. Resources are singletons registered with `ecs.insert_resource(...)` in `World::default` (`build_engine_ecs` in `src/ecs/world.rs`) and read as `Res<T>` / `ResMut<T>` or `world.res::<T>()`. Re-export systems in `src/systems.rs` and add to the prelude in `src/lib.rs`. 2024 edition, so no `mod.rs`.

## High-value gotchas

- `spawn_scene` overwrites global render settings (atmosphere included). Save and restore around a partial subtree spawn.
- Retained UI runs only when `UiPlugin` is composed (it installs the frame-schedule entry). Hide and show the tree at runtime with `world.res_mut::<RetainedUiState>().visible`.
- Renderer lifecycle, material reference counts, and transform propagation all derive from ECS change ticks and the structural change log. Write render-facing components through the stamping paths (`world.set`, `world.get_mut::<T>`, or `&mut` typed-query elements), never through raw-tier table writes, or the change goes unseen.
- `world.res_mut::<MeshRenderState>().request_full_rebuild()` remains the escape hatch if a batch ever looks stale after unusual churn.

You now have the map. Ask what to build, or state the task, then load the one or two docs and the reference app it touches.
