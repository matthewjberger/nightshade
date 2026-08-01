# The android input queue has one reader, so nightshade reads it before winit

Android hands an activity one input queue and one reader. Every event ends in
`AInputQueue_finishEvent`, so an event that anything reads is gone, and there is
no way to look at one and leave it for somebody else.

Winit reads that queue itself, in the same loop iteration, right after it calls
`NewEvents`. What it does with a motion event is turn it into a touch, whatever
sent it. A gamepad stick therefore arrived as a finger at the pointer's X and Y,
and the right stick, the triggers and the hat never arrived at all.

So nightshade takes the queue.
`nightshade_platform::commands::android_input::drain_input` runs from
`WindowContext::new_events`, which winit calls at the top of every iteration,
before its own read. It writes the touches and the keys into `Input` itself and
puts the gamepad events into `AndroidGamepadEvents`, which `nightshade-gamepad`
reads in `Stage::First` and the platform empties in `Stage::Last`. Winit then
finds the queue empty and sends nothing.

Three things follow from holding the queue:

- Every android key has to be named in that module. `to_key_code` maps the
  android keycode onto the winit `KeyCode` that `Input` is keyed by, because
  winit keeps its own map private.
- A pad that speaks as a keyboard is still a pad. A handheld with the controls
  built into it can send its face buttons without the gamepad bit in the source,
  so the `Button` keycodes count as a gamepad whatever the source says.
- The volume keys go back as `InputStatus::Unhandled`, which is what winit did
  with them, so the system still changes the volume.

`gilrs` reads no android device. On android it becomes a version whose
`Gilrs::new` always gives an error, so `nightshade-gamepad/Cargo.toml` keeps it
off the android build and takes the pad from `AndroidGamepadEvents` instead.
Android tells an app that a pad exists by sending input from it and never tells
an app that one went away, so a pad counts as connected from its first event
onward.
