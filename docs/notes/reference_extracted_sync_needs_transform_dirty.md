---
name: reference_extracted_sync_needs_transform_dirty
description: A render sync pulled out of scene_sync must gate on MeshRenderState transform_dirty as well as component change ticks, or moving entities freeze
type: reference
---

`collect_scene_dirt` in `nightshade-graphics/src/systems/scene_sync.rs` sets a domain's dirty flag
from four sources, not one: the component's own change ticks, the structural
log, the `visibility_changed` set, and **every entity in
`MeshRenderState::inner::transform_dirty` that carries the domain's
component**. A sync moved into its own plugin has to reproduce all four.

Missing the transform source is silent and looks like the feature is dead. A
scene that moves its lights every frame writes only `LocalTransform`, so a gate
on the `Light` component's ticks alone syncs once and then holds the renderer's
light table at the spawn positions while the emissive spheres keep flying. The
scene reads as unlit.

The structural log cannot be re-read by a second consumer, because
`collect_scene_dirt` drains it. Use a cursor plus `current_tick` for the
component ticks, an entity count for spawn and despawn, and `transform_dirty`
for movement. `nightshade-graphics/src/systems/light.rs` is the live example.
