---
name: reference_textured_material_needs_instanced_mesh
description: A textured material on a spawn_mesh_at mesh renders untextured, because only the instanced path carries the texture
type: reference
---

A material with `base_texture` applied to a mesh from `spawn_mesh_at` renders as
flat `base_color`, with no warning and no log line. The same material on
`spawn_instanced_mesh_with_material` draws the texture correctly. A material that carries a color and no texture draws the same on
both paths, so plain posts and water read as correct beside a wall that has
lost its texture.

The instanced path also repeats the texture by the instance scale, so a cube
scaled to 3 x 2 smears one 16 pixel tile across the whole face. Build from unit
cubes when you want one tile per face, the way the terrain blocks do.

Related: [[project_voxels_app]]
