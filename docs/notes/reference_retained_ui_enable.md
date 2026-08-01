---
name: reference_retained_ui_enable
description: The retained UI runs only where UiPlugin is composed, and RetainedUiRuntime.visible is the switch
type: reference
---

The retained UI runs only where `UiPlugin` is composed. `register_frame_systems`
puts `run_retained_ui_schedule` on `Stage::FramePostUpdate`, after
`update_global_transforms_system`, and `render_sync::publish_frame` on
`Stage::Last`, and the plugin sets `RetainedUiRuntime.visible` to true. An app
that never composes the plugin registers neither system and pays nothing.

`RetainedUiRuntime.visible` is the switch at run time. Every UI system reads it
and returns early when it is false, the layout, the picking, the color blend,
and the gamepad navigation included.

Build the tree once, in a startup system, with `UiTreeBuilder::new`, then
`.finish`. Keep the `Entity` handles it gives you in a resource and read
`ui_events` each frame. A runner that drives `State` itself registers the two
systems itself.
