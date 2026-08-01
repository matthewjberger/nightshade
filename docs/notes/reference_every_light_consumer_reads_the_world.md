---
name: reference_every_light_consumer_reads_the_world
description: Every light consumer in the renderer reads the world, and one that does not makes shadows vanish without a word
type: reference
---

Every consumer of the renderer's lights reads the same place, `configs.scene_world`,
because a light is a world entity. When one of them reads somewhere else the
shadows vanish without a word, because the shadow pass returns early when it
finds no directional, spotlight, or point light shadow. The consumers:

- `nightshade-render/src/queries/lights.rs`: `collect_lights`, `collect_area_lights`,
  and `build_render_lighting`. `nightshade-wgpu` reaches them through
  `renderer/wgpu/lights.rs`, which re-exports that module and nothing else.
- `wgpu/passes/shadow_depth/node.rs`: the directional cascade light in `prepare`,
  the point light candidates, and the `has_directional_shadow` gate in `execute`.
- `wgpu/passes/shadow_depth/atlas.rs`: `assign_spotlight_atlas`.
- `wgpu/passes/geometry/projection.rs`: `resolve_cookie_layers`,
  `resolve_area_emissive_layers`, and the point light shadow builder.

To see a shadow, capture a scene with low ambient and few lights, such as
`apps/features/lights`. A scene of eighty lights washes the shadows out.
