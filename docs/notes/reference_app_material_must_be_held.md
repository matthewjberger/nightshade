---
name: reference_app_material_must_be_held
description: A material an app registers is reclaimed unless it is held, so a mesh that names it draws white and untextured
type: reference
---

`cleanup_unused_resources_system` sweeps a range of material slots each frame
and removes any material no entity refers to. A world that comes down between
one map and the next leaves its materials unreferenced for a few frames, and a
mesh whose `MaterialRef` no longer resolves gets slot `0` in `material_sync.rs`
and draws with the default material, white and untextured. The sweep walks a
range with a cursor, so it can take one of an app's materials and leave the
next, which makes the fault intermittent and partial.

The registry therefore holds what an app registers:

- `insert_material` sets the `protected` flag on the slot it registers, and
  `material_registry_release` clears it. `GenerationalRegistry::protected` is a
  `Vec<bool>` beside `observed`, so the flag is reclaimed with the slot.
- `set_material` holds nothing. That material belongs to one entity, and
  tower_defense names one for each enemy, so holding them would leak.
- The sweep reads live slots as indexes, taken from the `MaterialRef` on each
  row. A reclaimed slot gets a new generation, so a stale handle fails the
  generation check rather than pointing at the material that replaced it.

The same empty frame makes the shadow pass take every mesh in the cache, which
overflows its 256 MB buffer. `RenderSettings::named_shadow_casters`
tells the pass the app names its casters, so an empty caster set means nothing
casts. voxels sets it.

Related: [[reference_textured_material_needs_instanced_mesh]]
