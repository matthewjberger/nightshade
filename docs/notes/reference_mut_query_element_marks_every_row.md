---
name: reference_mut_query_element_marks_every_row
description: A &mut element in a Query stamps a change tick on every visited row, so a wide &mut query makes the renderer resync thousands of entities each frame
type: reference
---

`Query<(&A, &mut B)>` stamps the change tick for `B` on **every row the query
visits**, not on the rows the closure writes. `crates/nightshade-ecs/src/ecs/query.rs`
does it in `par_fetch` (sets `peak_changed = current_tick`) and `par_item`
(`*cell = current_tick`), so `for_each_mut` marks the whole table.

Why it hurts: `scene_sync` reads `query_entities_changed_since` for `Visibility`,
`MorphWeights`, `RenderLayer`, `CullingMask`, `RenderMesh`, `BoundingVolume`, and
`MaterialRef`, and puts each changed entity into `object_refresh`,
`instanced_refresh`, and `object_state_dirty`. A system that scans thousands of
entities with `&mut Visibility` to flip a handful therefore makes the renderer
resync all of them, every frame. In apps/games/voxels that is most of a frame.

The fix: read with `&T`, collect the rows that actually change, and write them
through `commands.set`, which stamps only those. Take `&mut` in a query only for
components you write on nearly every visited row.
