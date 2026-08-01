---
name: reference_material_edit_per_frame_syncs_every_material
description: An edit to one material each frame makes the material sync, the mesh pass, and the shadow pass work over every material each frame, so a scrolling picture drifts on its texture transform
type: reference
---

`material_registry_mutate` marks the material dirty and the material sync moves
the material generation. Three consumers read that generation and each works
over the whole table: the sync rebuilds the id sets of every class, the mesh
pass converts every material and diffs the table, and the shadow pass converts
and uploads its masked materials again. The cost scales with how many materials
the app holds and never with the size of the edit, so one material edited every
frame makes every frame pay for all of them.

A picture that scrolls carries a `drift` on its `TextureTransform`, in texture
coordinates each second. The wgpu shaders move the coordinates by that drift
and the frame time, so water and lava scroll with no edit to the material. The
Vulkan backend does not read the drift, so the picture stands still there.
