---
name: reference_pre_render_runs_before_render_frame
description: A plugin's on_pre_render hook fires before render_frame, so anything it hands the renderer must already be complete
type: reference
---

`run_frame_body` calls `state.pre_render(renderer, world)` and then
`render_driver::render_frame(renderer, world)`. So every `App::on_pre_render`
hook runs **before** the frame driver's own preparation, not after it.

A hook therefore cannot hand the renderer data that the driver still has to
fill in. A capability that produces something the driver also touches owns the
whole production: the retained UI meshes its glyphs inside the hook, ahead of
the handoff, because `render_driver::extract` runs too late to shape them. The
tell for half a production is that rectangles draw and text draws empty.

Check what `render_driver/frame.rs` does between `pre_render` and the graph
execute before you decide a hook is late enough.

Both the winit loop and `tick_offscreen` go through `run_frame_body`, so a
`pre_render` hook does run in the worker path. See
[[reference_extracted_sync_needs_transform_dirty]].
