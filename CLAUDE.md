# Nightshade Engine

New to this repo, or starting a fresh session? Run `/intro-nightshade` for a full orientation: the two-layer architecture, where every doc lives, the reference apps to copy, the build/gate workflow, and how to work here without blowing context. The command is source-controlled at `.claude/commands/intro-nightshade.md`.

For games, start with the high-level `nightshade-api` facade. Read `docs/SPEC_RUST_API.md` (the full signature catalog) and the "Start Here" section of `docs/USAGE.md`. The api games `apps/coin_dash` and `apps/coin_rush` and the `template_api/` template are the working references. Reach for the engine layer only when the facade stops covering what you need.

Read `docs/USAGE.md` before writing or modifying engine code (the `nightshade` crate layer).

## Architecture

- ECS via `freecs::ecs!` macro. Components use bitflag constants: `CAMERA | LOCAL_TRANSFORM | GLOBAL_TRANSFORM`.
- `world.core` for ECS access (get/set components, queries). `world.resources` for singletons grouped into domain sub-structs (`physics`, `input`, `assets`, `text`, `loading`, `transform_state`, `cleanup`, `entities`, `commands`, `schedules`, etc.).
- Most systems run automatically via the frame schedule (`world.resources.schedules.frame`): physics, animation, navmesh, text sync, transforms, plus a single `RETAINED_UI` entry that runs the UI sub-schedule (`world.resources.schedules.retained_ui`). Don't call these manually.
- Camera controllers are opt-in: call `fly_camera_system(world)` or `pan_orbit_camera_system(world)` in `run_systems`. `fly_camera_system` skips if the active camera has `PAN_ORBIT_CAMERA`.
- Scenes load from glTF. Find entities by name after spawning (`world.core.find_entity_by_name`).
- `apps/sandbox` is the go-to reference for an idiomatic engine-level 3D game (when you have dropped below the `nightshade-api` facade). Read it before writing a new engine-layer app: state shell forwarding to system functions, own `freecs::ecs!` world, screen lifecycle in `systems/lifecycle.rs`, retained UI handles stored in resources, `theme.rs` for UI constants, prototype textures, level load/save. The `template/` crate is the same architecture in skeleton form.

## Querying

- Default: `world.core.query().with(MASK).iter(|entity, table, idx| { ... })` for reads, `query_mut()` for writes. Direct table access, no allocation, no per-entity getter cost.
- Use `.without(MASK)` for negative filters at table level instead of `.filter(|e| !world.core.entity_has_components(*e, MASK))`.
- For structural mutations during iteration (add/remove components, despawn, set component triggering archetype migration), use the two-pass pattern: closure-form gather into a small `Vec` of `(entity, payload)` tuples, then a second loop that applies the mutations.
- Two command queues at `world.resources.commands`: `commands.ecs` (DespawnRecursive, ReloadMaterial) is drained by `process_commands_system` in the frame schedule. `commands.render` (texture uploads, HDR skybox, IBL captures, screenshot) is drained by the renderer at frame setup. Push via `queue_ecs_command(world, EcsCommand::...)` or `queue_render_command(world, RenderCommand::...)` (free fns in the prelude, not methods).
- Escape hatches: `query_entities(MASK).collect::<Vec<_>>()` allocates per call; `for entity in query_entities(...)` doesn't allocate but pays per-entity getter overhead. Only use when the closure form can't return component references or when a helper requires `&World`.

## Dual-World Pattern

Games use two ECS worlds: the engine `World` (rendering, physics, transforms) and a user-defined `GameWorld` via a second `freecs::ecs!` (game logic, AI, state). The engine ships an `EngineEntity(Entity)` component (`nightshade::ecs::sync::EngineEntity`) and sync helpers (`sync_engine_translation`, `sync_engine_transform`, `despawn_linked`). Users include `EngineEntity` in their own `freecs::ecs!` block to link game entities to their engine-side render entities, then sync positions from game world to engine world in a render_sync pass. See `apps/game/src/systems/props.rs` for the live pattern.

## Gotchas

- WGSL uniform structs need 16-byte alignment. Use `vec4` packing, not `vec3`.
- `nalgebra_glm` only, never `glam`. `Vec2::component_mul()` for element-wise multiply.
- `mark_local_transform_dirty(world, entity)` after manually setting transforms.
- `freecs::Entity` is the entity type everywhere.
- Sprite atlas slot size: `nightshade::render::SPRITE_ATLAS_SLOT_SIZE` = (512, 512).
