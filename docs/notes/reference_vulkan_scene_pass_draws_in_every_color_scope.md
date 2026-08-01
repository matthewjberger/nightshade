---
name: reference_vulkan_scene_pass_draws_in_every_color_scope
description: A Vulkan pass installed at PassStage::Scene has its draw called once in every color scope, so a pass that draws in one of them must gate on record.scope
type: reference
---

The Vulkan frame walks a list of scopes, and for each one it calls `draw` on
every pass whose stage the scope names. `PassStage::Scene` is named by the
`Color`, `Refracted`, `Blended`, and `OverSolids` scopes, so a scene pass has
its `draw` called four times a frame, into four different sets of attachments.
The `Blended` scope draws into the accumulated and revealed targets, whose
formats differ from the lit color target a pipeline is usually made for.

**How it fails when you get it wrong:** the draw records into a scope whose
attachment formats the pipeline was not made for. Nothing in the frame
refuses it, and what shows depends on the driver: a blend that lands where it
should not, a sprite that shows through the solids, or nothing visible at all.

**How to apply:** a scene pass that belongs in one scope returns early from
`draw` unless `record.scope` is that scope. The sprite pass draws only in
`ScopeKind::OverSolids`, the scope that loads the lit color and the settled
depth, which is where a quad that tests against the scene and blends over it
belongs. The text pass gates on `ScopeKind::Color` the same way. Do the
uploads and the buffer writes in `compute`, which runs once a frame before
any scope opens, so `draw` only binds and draws.
