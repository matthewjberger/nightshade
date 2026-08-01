---
name: project_input_capture_resource
description: InputCapture arbitrates the pointer and the keyboard through claims keyed by the type that makes them
type: project
---

`InputCapture` arbitrates the pointer and the keyboard between everything that
wants them. Claims are keyed by the **type that makes them**, not by named fields:

```rust
capture.set_pointer::<MyOverlay>(is_hovering);
capture.set_keyboard::<MyOverlay>(is_typing);
if capture.pointer { /* something owns the pointer */ }
```

Read through `pointer` / `keyboard` / `any`, never a field. Each claimant
writes only its own slot every frame, passing the condition directly rather than
calling only when it holds, so claims release on the frame the condition stops
and run order stops mattering.

**Why:** one shared field cannot hold two claimants. Whichever writes it last
decides the answer, and a claimant that only sets it true never releases what
it takes.

The offscreen viewport texture requests (`required_cameras`,
`required_camera_sizes`, `viewport_texture_sizes`) sit in `Viewport`, beside the
camera tile rects, not here.
