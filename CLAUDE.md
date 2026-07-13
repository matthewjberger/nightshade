# Nightshade Engine

New to this repo, or starting a fresh session? Run `/intro-nightshade` for a full orientation: the two-layer architecture, where every doc lives, the reference apps to copy, the build/gate workflow, and how to work here without blowing context. The command is source-controlled at `.claude/commands/intro-nightshade.md`.

For games, start with the high-level `nightshade-api` facade. Read `docs/SPEC_RUST_API.md` (the full signature catalog) and the "Start Here" section of `docs/USAGE.md`. The api games `apps/coin_dash` and `apps/coin_rush` and the `template_api/` template are the working references. Reach for the engine layer only when the facade stops covering what you need.

Read `docs/USAGE.md` before writing or modifying engine code (the `nightshade` crate layer).

## Architecture

- ECS via `freecs::dynamic` (no `ecs!` macro in the engine). `World { ecs: DynEcs, resources: Resources }` with two member worlds: `world.ecs.worlds[CORE]` (scene) and `world.ecs.worlds[UI]` (retained UI), declared with `freecs::dynamic_worlds!`. Components register eagerly in declaration order in `src/ecs/world.rs` (`freecs::dynamic_schema!`), which also emits the bitflag constants (`CAMERA | LOCAL_TRANSFORM | GLOBAL_TRANSFORM`); registration order is the schema, never reorder it.
- Typed access is group-routed: `world.get::<Camera>(entity)` / `world.get_mut::<T>` / `world.set(entity, value)` / `world.has::<T>` resolve the owning member world per component type (get_mut and set stamp change ticks; set never registers a type lazily, placement is a schema decision). Group operations (spawn_count, spawn_with, despawn_entities, despawn_recursive, is_alive) are on `world` directly through deref to `DynEcs`. Member indexing (`world.ecs.worlds[CORE]`) remains for world-level operations: pinned per-world queries, ticks, structural logs, raw table passes, snapshots. `world.resources` for singletons grouped into domain sub-structs (`physics`, `input`, `assets`, `text`, `loading`, `transform_state`, `cleanup`, `entities`, `commands`, `schedules`, etc.).
- Most systems run automatically via the frame schedule (`world.resources.schedules.frame`): physics, animation, navmesh, text sync, transforms, plus a single `RETAINED_UI` entry that runs the UI sub-schedule (`world.resources.schedules.retained_ui`). Don't call these manually.
- Camera controllers are opt-in: call `fly_camera_system(world)` or `pan_orbit_camera_system(world)` in `run_systems`. `fly_camera_system` skips if the active camera has `PAN_ORBIT_CAMERA`.
- Scenes load from glTF. Find entities by name after spawning (`world.ecs.worlds[CORE].find_entity_by_name`).
- `apps/sandbox` is the go-to reference for an idiomatic engine-level 3D game (when you have dropped below the `nightshade-api` facade). Read it before writing a new engine-layer app: state shell forwarding to system functions, an app member world for game components, screen lifecycle in `systems/lifecycle.rs`, retained UI handles stored in a plain resources struct, `theme.rs` for UI constants, prototype textures, level load/save. The `template/` crate is the same architecture in skeleton form.

## Querying

- Default: typed tuple queries. `world.query_ref::<(&Cloth, &RenderMesh)>().iter()` for reads (a real iterator on `&world`, never registers types), `query::<(&mut A, &B)>().for_each(...)` for writes (`&mut` elements stamp change ticks per visited row, so take `&mut` only for what you actually write). Filters: `.with::<T>()` / `.without::<T>()` / `.with_mask(MASK)` / `.without_mask(MASK)` / `.changed::<T>()` / `.added::<T>()`. Up to eight elements, `Option<&T>` for optional presence.
- Bulk visit-don't-write passes stay on the raw tier (`for_each_mut(MASK, 0, |entity, table, idx| ...)` with `table.column_mut(key)`, keys from `register::<T>()`), which does not stamp ticks; pair with explicit side-channel marks or `mark_changed`/`mark_columns_changed`.
- Entity-only iteration: `query_entities(MASK)` (allocation-free iterator).
- For structural mutations during iteration (add/remove components, despawn, set component triggering archetype migration), use the two-pass pattern: gather into a small `Vec` of `(entity, payload)` tuples, then a second loop that applies the mutations.
- Two command queues at `world.resources.commands`: `commands.ecs` (DespawnRecursive, ReloadMaterial) is drained by `process_commands_system` in the frame schedule. `commands.render` (texture uploads, HDR skybox, IBL captures, screenshot) is drained by the renderer at frame setup. Push via `queue_ecs_command(world, EcsCommand::...)` or `queue_render_command(world, RenderCommand::...)` (free fns in the prelude, not methods).

## App Member-World Pattern

Games put their components in a third member world of the engine group, sharing entity identity with the render entities. In the app's `ecs.rs`: `pub const GAME: usize = 2;` plus `freecs::dynamic_schema! { pub fn register_<app>_components { field: Type => CONST, } }`, registered once at initialize with `world.ecs.add_world_at(GAME, register_<app>_components());`. Game state is set directly on the engine entity (`world.set(entity, component)` routes to the member world and creates the row), read back with routed typed queries, and one recursive despawn tears down render and game state together (group despawn retires rows in every member world). App-wide state lives in a plain resources struct owned by the app's `State` type, not in an ECS world. There is no link component and no position sync pass; write `LocalTransform` through `world.get_mut` directly. See `apps/mini_dungeon` or `apps/tower_defense` for live examples and `template/` for the skeleton.

## Gotchas

- WGSL uniform structs need 16-byte alignment. Use `vec4` packing, not `vec3`.
- `nalgebra_glm` only, never `glam`. `Vec2::component_mul()` for element-wise multiply.
- The renderer syncs by diffing freecs change ticks, and raw-tier table writes (`for_each_mut`, `for_each_tables_mut`, `column_mut`) do not stamp them. Write every render-facing component (transforms, visibility, lights, morph weights, text, water, decals, instanced meshes) through the stamping paths (`set`, `get_mut::<T>`, or `&mut` typed-query elements) or the change never reaches the renderer. Raw-tier bulk passes can opt in afterward with `world.ecs.worlds[CORE].mark_changed(entity, MASK)` or per-table `table.mark_columns_changed(MASK, tick)`. Mutate registered materials through `material_registry_mutate`, never through `registry_entry_by_name_mut`.
- `freecs::Entity` is the entity type everywhere.
- Sprite atlas slot size: `nightshade::render::SPRITE_ATLAS_SLOT_SIZE` = (512, 512).
