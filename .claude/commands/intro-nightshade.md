---
description: Orient a new session on the nightshade engine. Architecture, where every detail lives, and how to work here without blowing context.
---

# Orient: Nightshade Engine

You are starting work in the nightshade engine repo. This is your orientation. Read it in full and build the mental model below. `CLAUDE.md` carries the rules and the architecture, `docs/notes/` carries one trap or app fact to a file, `docs/antipatterns.md` lists the shapes this repo refuses, and the source carries the rest. The source holds no comment of any kind, so the names and the bands are what tell you what a thing is. This file is the map. Load the territory only as a task needs it.

## What this is

A cross-platform data-oriented game engine (wgpu, nightshade_ecs, nalgebra_glm). Every crate carries `publish = false`.

- **`nightshade-ecs`** (`crates/nightshade-ecs`) is the storage: the world, the components, the queries, the resource map, the schedules, and the state machine.
- **`nightshade`** (`crates/nightshade`) sits on it and holds `App`, `Plugin`, and `Stage` in three files. It carries no capability and no GPU API.
- **The forty plugin crates** (`crates/plugins/`) hold every capability, `nightshade-platform` and `nightshade-scene` among them. They sit in four levels a crate may only name downward: `base/` names the ECS and nothing else, `describe/` is the platform, scene, render, text and paint chain every other crate speaks, `domain/` holds one capability each, and `compose/` holds the six roots that run a loop or own a backend. A game composes the ones it wants.

## Step 1: read before writing anything

Read the app closest to the task. `apps/games/tower_defense` is the canonical engine-layer game, `template/` is the same architecture stripped to a skeleton, `apps/rendering/` holds one app for each capability of the renderer, and `apps/vulkan/` the three that only Vulkan can draw.

Read `docs/antipatterns.md` in full before you write a struct, a system signature, a random draw, or a tuning number. It is the list of shapes this repo refuses, with the shape that replaces each one, and `just lint-dag` fails on several of them.

## The mental model (hold this, do not re-derive it)

- **A program is composed of plugins.** `App::new().add_plugins(DefaultPlugins).add_plugin(MyGamePlugin).run()`. A `Plugin`'s `build` inserts resources and registers systems onto the `Stage` enum, and `add_system` takes both a plain `fn(&mut World)` and a parameter system. A capability that has something to say to another publishes types in its own `contract` feature rather than reaching across.
- **ECS is nightshade_ecs.** There is one `World`, `{ storage, events, resources }`, and it derefs to the storage for everything. That one world holds every component: the scene, the renderer's, which reads that same world, the retained UI's, each composed capability's, and the app's own.
- **Components are types, not bits.** Nothing declares a component, and nothing registers, marks, or lists one. Any `Send + Sync + Default + 'static` type is a component, and the first write of it appends it to the world's schema and takes the next of 256 mask slots. The ECS is runtime storage: it has no snapshots, no deltas, and no codecs, so no component is named anywhere. Nothing names a mask slot. Every selection API takes the component types as a generic bundle tuple: `query_entities::<(Camera, LocalTransform)>()`, one-element tuples keep the trailing comma, `()` means no components. Point access is typed: `world.get::<Camera>(entity)`, `world.get_mut::<T>`, `world.set(entity, value)`, `world.has::<T>`. The default single-entity spawn is a tuple, `world.spawn((component, ...))`, deriving the archetype from the component types; `spawn_entities::<(A, B)>(count)` covers batches and all-default rows.
- **Resources are singletons in the ECS resource map.** Read them as `Res<T>` / `ResMut<T>` system parameters, or `world.res::<T>()` / `world.res_mut::<T>()` inside a `&mut World` system. Both accessors panic when the resource is absent, so a capability whose plugin was never composed fails loudly.
- **The default query is a typed tuple.** `query_ref::<(&A, &B)>().iter()` on `&world` for reads, `query::<(&mut A, &B)>().for_each(...)` for writes (`&mut` elements stamp change ticks per visited row, so take `&mut` only for what you write), with `.with::<T>()` / `.without::<T>()` / `.changed::<T>()` / `.added::<T>()` for filters and `Option<&T>` for optional presence. Systems also take `Query<...>`, `Res<T>`, `ResMut<T>`, `EventReader<T>`, and `ParamSet` parameters. For structural mutation during iteration, gather `(entity, payload)` into a `Vec`, then apply in a second pass.
- **Most engine systems run automatically** in the frame stages (`Stage::FrameStart`, `Stage::FrameUpdate`, `Stage::FramePostUpdate`, `Stage::Render`, `Stage::Last`) that follow the game stages each frame in one `Stages` run: the core systems land first and each composed plugin registers its own with `app.add_system(Stage::..., system)`; within a stage, systems run in registration order. Do not call these manually. Camera controllers are opt-in, either through `CameraControllerPlugin` or called from the game plugin's update systems. Do not compose that plugin in a game that drives its own camera: its frame systems move whichever camera is active, so WASD flies it and a right drag turns it, and no marker component opts out. Call the controller function you want from your own system instead. Never hand-run an engine system from app code. Fix the data dependency instead.
- **Game components.** An app adds nothing to get its own components. Game state is set directly on the engine entity with `world.set(entity, component)`, which takes a mask slot the first time it sees the type, so game code stores components on engine entities without declaring anything but the struct. A game that saves writes its own format over the data it authored, because the ECS carries no serialization. Entity identity is shared with the scene entities, so one recursive despawn tears down both. App-wide state is a plain resources struct the plugin inserts with `app.insert_resource(...)`, read back as `Res<T>` / `ResMut<T>`. `template/` is the starting point to copy, and `apps/games/tower_defense` is the same shape at full size.
- **Rendering is a description and two backends.** `nightshade-render` holds the data both backends read: materials, meshes, textures, lights, the render config, the frame description, the per-entity render components, and the math that is the same whichever backend draws. It holds no system, no plugin, and no GPU API. `nightshade-wgpu` is the wgpu backend, which owns the render graph and every pass and shows two paths, `nightshade_wgpu::wgpu` and `nightshade_wgpu::backend`. `nightshade-vulkan` is the sibling backend for the mesh shaders and the two ray traced apps. A capability that draws publishes a frame into a contract crate, a `PaintFrame`, a `SpriteFrame`, a `DecalFrame`, or a `ParticleFrame`, and names no GPU API. The engine renders reverse-Z, so a backend clears depth to zero and compares with `GREATER_OR_EQUAL`.
- **The window, the input, and the clock are `nightshade-platform`.** It is data the way `nightshade-render` is data: the `Window`, `Input`, `Time`, and `InputCapture` resources and `AppEvent`, with nothing that drives them. `nightshade-window` owns the winit loop over it, carries `WindowPlugin`, which composes the platform, the scene, and the graphics, and names no GPU API of its own. A capability that wants the window or the input names `nightshade-platform` directly, and one that wants a transform or a name names `nightshade-scene`.
- **The retained UI is `nightshade-ui`,** a plugin crate like the rest, composed as `nightshade_ui::UiPlugin::new()`. It builds on `nightshade-ecs`, the platform and scene contracts, and the text and paint crates, and it publishes a `PaintFrame` into `nightshade-paint` that the backend draws. `just lint-dag` enforces the layering. `apps/ui/gallery` calls almost every widget constructor, so a dead-code sweep of the UI measures that app rather than the crate.

## Reference code (copy proven patterns, do not invent new architecture)

- `apps/games/tower_defense`: the canonical engine-layer 3D game. A plugin in `plugin.rs` registers startup and update systems onto free functions under `systems/`, its own game components (`components.rs`), screens driven by app states (`insert_state` / `on_enter` / `while_in` / `next_state`), and its retained UI under `retained_ui/`. Read it before writing a new engine-layer app.
- `template/`: the same architecture stripped to a starting point.
- `apps/games/coin_dash`, `apps/games/coin_rush`: idiomatic facade games. State in a plain struct holding `Entity` handles, laid out as `components` / `resources` / `systems` / `theme`.
- `apps/rendering/`: one app for each capability of the renderer, each laid out like `template/`, composing a backend over a Khronos sample model it fetches over the web, with its README saying what to look for. They open on wgpu and take `--no-default-features --features vulkan` to open on the other backend. Copy one of these to demonstrate a renderer feature.
- `apps/vulkan/`: the three that only Vulkan can draw, mesh shaders and the two ray-traced ones, laid out the same way.
- `apps/ui/gallery`: every retained-UI widget constructor on one screen. It is the only caller of most of them, so a dead-code sweep of the UI measures this app rather than the engine.
- `apps/features/`: twenty-one smaller apps, each on one capability or one scene. Among them `fireworks` (particles, and the app `just run-web` serves), `navmesh`, `picking`, `sprites`, `lights`, `sdf` (a world of signed distance fields cached in a clipmap of sparse bricks), `sponza` (the scene the flicker pair is captured from), and `boids_cpu` with `boids_gpu`, the same flock on the cpu and on the gpu.

## How to work here (the method that protects context)

1. **Delegate broad searches.** For "where is X", "who calls Y", or any sweep across many files, spawn an `Explore` agent and keep only its conclusion. Do not read a pile of files into your own context.
2. **Read the notes in `docs/notes/` that touch the module you are changing.** The source carries no comment, so the design intent lives in the notes and in `CLAUDE.md`.
3. **Read whole files before editing them.** Otherwise you duplicate code or misread the architecture.
4. **Reuse existing patterns.** Find the nearest working app or example and follow it. Respect the `systems/<name>.rs` architecture: behavior goes in a system module, not lumped into a state file.
5. **When a search drives a removal or refactor**, double-check for other consumers across all apps before declaring it done.

## Hard rules (non-negotiable, full set in the root `CLAUDE.md` and the user's global config)

- Rust: no comments ever, rustdoc included, and `just lint-dag` fails on one, no debug logging, no abbreviations (`index`, not `i`), no `unsafe` unless told, no `mod.rs` (2024 edition), no `#[allow(...)]` to silence a lint (fix the cause), no `Rc<RefCell<T>>`.
- Every capability is a crate under `crates/plugins/<level>/`, laid out in the same bands. A crate that needs another takes it with `default-features = false` and its `contract` feature, never its `plugin`.
- `nalgebra_glm` only, never `glam`. Use `Vec2::component_mul()` for element-wise multiply. WGSL uniform structs need 16-byte alignment, so pack with `vec4`, not `vec3`.
- Data-oriented, not OOP. No "Manager" suffixes, no one-line forwarding wrappers, no gratuitous `pub`. The builder pattern is the lone allowed OOP-ish exception.
- No wall of tuning constants. A value a person is meant to change is declared once in a `nightshade_config::config!` block and read through `Res<Config<Settings>>`, so a panel can show it and a file can keep it. A `pub const` is for what the program is made of: a table, an asset, a structural limit like a tile size or a buffer cap.
- When you replace a function, delete the old one and its now-orphaned helpers and imports in the same change. Never leave both.
- Never default a new capability off. Build it as THE path, not a flag that ships false. Cargo features are the exception: a capability plugin is a feature and stays off by default.
- Transform writes through `world.set(entity, LocalTransform { .. })` or `world.get_mut::<LocalTransform>(entity)` propagate automatically via ECS change ticks. `nightshade_ecs::Entity` is the entity type everywhere.
- Windows host: use PowerShell-compatible commands and the dedicated file tools, not the Bash tool, for path operations.

## Build, run, and the commit gate

Use the `justfile`, never raw `cargo run`.

- `just run <project>` runs a native app by package name (blank arg opens the sandbox). `just search` picks one from a list of every app. `just run <project> <args>` and `just run-app <project> <args>` both pass args straight to the app, with no `--` of your own.
- Every app composes `CliPlugin`, so every app takes `--capture <path>`, `--capture-next <path>`, `--capture-frame <n>`, `--capture-settle <n>`, `--size WxH`, and `--help` through `nightshade-cli`. An app's own flags are a `clap::Args` struct in its `resources.rs`, composed as `CliPlugin::<Arguments>` right after `WindowPlugin`. Never read `std::env::args` in an app.
- `just run-wasm <project>` does a browser build through Trunk (`fireworks` unless one is given). `just run-vr <name>` needs a desktop OpenXR runtime and a headset.
- Gate what your change reaches, as the **Gates** paragraph of `CLAUDE.md` says: `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` always, then the `just lint-*` check the change reaches, then `cargo test --lib -p <crate>` for the crates you touched. Leave the whole `just lint` and `just test` to CI. `just lint` is clippy over the workspace with warnings as errors plus eight checks the compiler cannot see: `lint-wasm`, `lint-crates-isolated`, `lint-apps-isolated`, `lint-features`, `check-feature-forwarding`, `lint-dag` (the module DAG and the no-cross-capability rule), `lint-shaders` (composes every WGSL with naga_oil and counts each stage against the device budget), and `lint-vulkan-shaders`.
- In PowerShell 5.1 do not pipe `2>&1` from cargo or just. It false-fails `$LASTEXITCODE`.
- Git only when explicitly told. Conventional Commits (`type(scope): description`). No em-dashes, semicolons, `@`, or attribution or Co-Authored-By lines in messages. Never push without the user verifying.

## Adding a component or system

- **App-level game data** does not touch engine source. Set it on the engine entity with `world.set(entity, component)` and the type takes a mask slot on that first write. A game that saves writes its own format. See `template/src/components.rs`. App-wide singletons go in a plain struct inserted with `app.insert_resource(...)` and read as `Res<T>` / `ResMut<T>`.
- **A capability** is its own crate under `crates/plugins/<level>/`, carrying the plugin that installs it. The bands are the same in every one: per-entity data in `components`, shared data in `resources`, tables in `data`, frame behavior in `systems`, changes in `commands`, reports in `queries`, and `plugin.rs` holding only its plugin struct, that struct's `Plugin` impl, and functions named `install`, `register` or `initialize`. `just lint-dag` fails on a cycle between them, on a reference to another capability, and on a crate that names a level above its own. Composing the plugin is what activates it.
- **A component** is the struct that declares it. Nothing registers it: the type takes a mask slot on its first write. Put it in the crate's `components` band and re-export it in that crate's prelude. Resources are inserted with `insert_resource` and read as `Res<T>` or `world.res::<T>()`. 2024 edition, so no `mod.rs`.

## High-value gotchas

- Retained UI runs only when `UiPlugin` is composed (it installs the frame-schedule entry). Compose it as `nightshade_ui::UiPlugin::new()`. Hide and show every tree at runtime with `world.res_mut::<RetainedUiRuntime>().visible`. `UiTreeBuilder::new(build.context())` builds one tree and can be called again for a second, which is how a head-up display and a menu board hide apart: `ui_set_visible` takes `RetainedUiDirty` and a node entity, not a tree root, so keep the handle to the panel you built.
- Renderer lifecycle, material reference counts, and transform propagation all derive from ECS change ticks and the structural change log. Write render-facing components through the stamping paths (`world.set`, `world.get_mut::<T>`, or `&mut` typed-query elements), never through raw-tier table writes (`for_each_mut`, `column_mut`), or the change goes unseen. A raw-tier pass can opt in afterward with a per-table `table.mark_column_changed(key, tick)`, one call per component, with a `ComponentKey<T>` from `world.component_key::<T>()`.
- App states drive screen flow: `app.insert_state(Screen::Title)` once, `app.on_enter(state, systems)` to build a screen, `while_in(state, (systems, ...))` to run one, and a parameter system asks for the next screen with `ResMut<NextState<S>>`. One tuple holds at most 16 systems, so split a long screen into two registrations and keep the order. A transition into the state that is already current fires no event, so `on_enter` does not run and an in-place rebuild has to run from a flag of your own.
- `world.res_mut::<MeshRenderState>().request_full_rebuild()` remains the escape hatch if a batch ever looks stale after unusual churn.
- naga_oil rejects a WGSL struct member whose name ends in a digit, and it fails at composition, which is run time. The compute stage caps at 8 storage buffers.

You now have the map. Ask what to build, or state the task, then load the one or two docs and the reference app it touches.
