---
name: reference_outline_needs_debug_render_feature
description: The outline mask and outline passes sit behind nightshade-render's debug_render cargo feature, which is not on by default
type: reference
---

The stencil outline (the orange Blender-style highlight) is drawn by
`OutlineMaskPass` and `OutlinePass`, and both are added to the render graph
only under `#[cfg(feature = "debug_render")]` in
`crates/plugins/compose/nightshade-wgpu/src/renderer/wgpu/initialization.rs`.
`debug_render` is a feature of **`nightshade-render`** and is not in its
defaults. `apps/features/picking`, `apps/features/boids_cpu`, and
`tools/shader-check` enable it by name.

So setting `Outline.enabled = true` and filling `Outline.active_entity` or
`Outline.entities` does nothing on a default build. The passes are not in the
graph at all. Add `nightshade-render = { workspace = true, features =
["debug_render"] }`.

Once the feature is on, the mask still gates twice more: it reads
`active_view.outline_enabled`, derived per camera through `EffectiveShading`,
and that is forced off when the camera's `ViewportShading.show_overlays` is
false. The mask seeds from `active_entity` plus `entities` and expands each to
its descendants, so selecting a parent outlines its mesh descendants.

The resource is `Outline`, with `enabled` and `color`. The component is
`Outlined`, the pass is `OutlineMaskPass`, and the shader is
`outline_mask.wgsl`.

The same `debug_render` flag gates the other debug overlays, the bounding
volumes and the normals.
