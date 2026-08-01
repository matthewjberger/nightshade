---
name: reference_close_request_runs_one_frame
description: A close request from the window sets Window.close_requested and runs one more frame before the loop exits, so work that must land on the disk before the app closes runs in that frame
type: reference
---

The winit runner in `nightshade-window` handles a `CloseRequested` event by
setting `close_requested` on the `Window` resource and nothing else. The next
`RedrawRequested` event runs a whole frame with that flag set, and the runner
sets `should_exit` after that frame and leaves the loop. A system in
`Stage::Update` that reads `close_requested` therefore runs exactly once with
it set, which is where a save on exit belongs. `apps/games/voxels` reads it
in `keep_the_world`.

`should_exit` is different. A system that sets it in the frame body ends the
app after that frame, with no frame in between, so a save that must follow a
button that quits runs in the same stage as the button, after it in
registration order.

No frame runs between the close event and the exit for a system that reads
`should_exit` alone, so a save gated on `should_exit` never runs when the
window is closed from its own close button.
