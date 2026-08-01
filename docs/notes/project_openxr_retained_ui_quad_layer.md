---
name: project_openxr_retained_ui_quad_layer
description: The retained UI reaches the headset as an OpenXR quad layer through a renderer paint target that redirects the paint passes, and the paint prepare consumes its draw list, so a second paint run in one frame draws nothing
type: project
---

`apps/features/jarvis` is the reference VR app: a helmet HUD on the quad layer, gauntlet flight, repulsor bolts, lock-on missiles, and fireworks-style particle bursts.

The retained UI shows in a headset as one `CompositionLayerQuad`, not as a draw into each eye. `WgpuRenderer.paint_target` (set by `set_external_paint_target`) makes `dispatch_window_renders` bind that image to the `swapchain` resource, clear it to transparent, run the swapchain-stage passes (paint and paint image) into it, and rebind the surface. The OpenXR runner hands it the acquired image of a one-layer UI swapchain the size of the desktop window, and `XrSession::end_frame` submits the quad after the projection layer with `BLEND_TEXTURE_SOURCE_ALPHA`. `XrUiPanel` (contract) holds the anchor, distance, width, and pointer hand. `ui_pointer_system` at `Stage::First` turns the aim pose ray into `Input.mouse` while it hits the panel. `XrUiPointer` publishes the hit.

**Why:** The paint passes are swapchain-stage passes and the per-camera loop disables them, so the eyes never see the UI. The paint prepare takes its draw list with `mem::take`, so running the paint passes twice in one frame draws nothing the second time. That is why the paint is redirected and not duplicated: the desktop mirror shows the scene without the UI while the panel is enabled. The paint pipelines use `ALPHA_BLENDING` over a transparent clear, which yields premultiplied alpha, so the quad must not carry `UNPREMULTIPLIED_ALPHA`.

**How to apply:** to show the UI on the mirror as well, make the paint prepare keep its draw list until a new one replaces it, then run the passes twice. `ERROR_FORM_FACTOR_UNAVAILABLE` at start means the headset is asleep or unplugged, and a missing `CameraPlugin` panics at `spawn_xr_rig` on `ActiveCamera`.
