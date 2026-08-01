---
name: reference_wasm_dpr_pointer_mismatch
description: On wasm the canvas uses a capped device-pixel ratio but winit reports pointer coords at the raw ratio, and the two must be reconciled
type: reference
---

On the web, `nightshade-window` sizes the canvas backing buffer (and `cached_viewport_size`) with `wasm_capped_dpr` in `pacing.rs`, which caps the device-pixel ratio (1.5 when the smaller CSS dimension is below 700px, else 2.0). But `cached_scale_factor` is set from winit's raw `scale_factor` (the uncapped `devicePixelRatio`), and winit reports touch and cursor positions at that raw ratio (`css * raw_dpr`). The UI is laid out and hit-tested in `cached_viewport_size` space (`css * capped_dpr`).

When the cap bites (raw != capped), pointer input lands offset from where the UI is drawn, the error growing with distance from the top-left. A phone 375 CSS wide is capped to 1.5 against a raw 2.0, so every touch lands 1.33 times out. A tablet 820 CSS wide is never capped and lines up exactly.

The fix: `Window.pointer_viewport_scale = capped/raw` (1.0 on native and whenever uncapped), set wherever the canvas is sized (resumed + Resized), and multiplied into every incoming winit pointer coordinate (`handle_touch_input` touch.location and the `CursorMoved` position, both in `platform/input.rs`) so pointer input shares the backing-buffer space. If you ever add a new pointer-coordinate source, apply this factor too. See also [[reference_retained_ui_touch_clicks_fragile]] and [[reference_absolute_scale_three_paths]].
