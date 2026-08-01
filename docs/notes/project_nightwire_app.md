---
name: project_nightwire_app
description: apps/games/nightwire is the interactive fiction app that is retained UI only, with a toggle-able CRT pass
type: project
---

`apps/games/nightwire` is a psychological horror interactive fiction game: one
night on a 1977 valley switchboard. It has no 3D scene. `initialize` sets
`render_world_to_swapchain = false`, spawns one orthographic camera, and the
whole game is the retained UI, so the same picture runs on the desktop, on the
web, and on android.

The slice is three calls, a notebook of names you raise with a later caller,
one deputy to send or hold, and a shift report. The transcript types itself in
a letter at a time from a `VecDeque<String>`, and the script is authored as
short pre-wrapped lines in `data.rs` so fixed row slots need no text wrapping.
Per-entity slot indexes (`LineSlot`, `ReplySlot`, `CardSlot`, `SummarySlot`)
are components, and only the singleton node handles sit in resources.

`glass.rs` is the CRT pass, modelled on `apps/games/iam/src/screen.rs`: it
redirects `viewport_compose_pass`, `paint_pass`, and `paint_image_pass` into a
transient texture with `render_graph_bind_pass_slot`, then adds a
`PassStage::Swapchain` pass that reads it. `F1` or a notebook button fades the
effect with one `amount` uniform, so nothing is added to or removed from the
graph at run time.

Flags: `--shift` opens on the board, `--no-glass` opens without the effect.
