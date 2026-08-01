---
name: reference_input_reset_in_frame_schedule
description: Edge-triggered input is cleared inside the frame schedule, so code that runs outside run_systems reads it already cleared
type: reference
---

The engine clears edge-triggered input at the end of the frame. `end_frame` on
the keyboard drops the just-pressed and just-released keys and the frame's
characters, `end_frame` on the mouse drops the just-pressed and just-released
buttons and the position, wheel and raw deltas, `end_frame` on the touch drops
the frame's touches. The three systems
that call them, `reset_keyboard_system`, `reset_mouse_system`, and
`reset_touch_system`, are registered by
`register_frame_systems` in `nightshade-platform`'s `plugin.rs` on
`Stage::Last`, so they run at the end of the frame schedule that
`State::run_systems` drives. Held state survives: `is_key_pressed` and the
mouse pressed set read the same after the reset as before it.

A driver that runs code of its own outside `run_systems` therefore reads input
that is already cleared. Such a driver registers those systems itself at the
top of its own frame instead, which keeps the meaning of a frame body and lets
the flags live across its gap.

A capture with no key press cannot catch this. Only a hand test with input can.
