# Nightshade Engine

Read `docs/USAGE.md` before writing or modifying engine code.

## Architecture

- ECS via `freecs::ecs!` macro. Components use bitflag constants: `CAMERA | LOCAL_TRANSFORM | GLOBAL_TRANSFORM`.
- `world.core` for ECS access (get/set components, queries). `world.resources` for singletons (physics, input, audio, window, etc.).
- Most systems run automatically via the frame schedule (`world.resources.frame_schedule`): physics, animation, camera controllers, navmesh, text sync, lattice, UI, transforms. Don't call these manually.
- Camera controllers are component-driven: `fly_camera_system` runs unless the active camera has `PAN_ORBIT_CAMERA`, in which case `pan_orbit_camera_system` runs instead.
- Scenes load from glTF. Find entities by name after spawning (`world.core.find_entity_by_name`).

## Dual-World Pattern

Games use two ECS worlds: the engine `World` (rendering, physics, transforms) and a user-defined `GameWorld` via a second `freecs::ecs!` (game logic, AI, state). An `EngineEntity(Entity)` component links game entities to their engine-side render entities. Sync positions from game world to engine world in a render_sync pass.

## Gotchas

- WGSL uniform structs need 16-byte alignment. Use `vec4` packing, not `vec3`.
- `nalgebra_glm` only, never `glam`. `Vec2::component_mul()` for element-wise multiply.
- `mark_local_transform_dirty(world, entity)` after manually setting transforms.
- `freecs::Entity` is the entity type everywhere.
- Sprite atlas slot size: `nightshade::render::SPRITE_ATLAS_SLOT_SIZE` = (512, 512).
