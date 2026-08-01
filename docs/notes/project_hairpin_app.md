---
name: project_hairpin_app
description: apps/games/hairpin, the arcade rally with drift and lap times
type: project
---

`apps/games/hairpin` is an arcade rally that stands beside Micro Machines and Art of Rally.

The vertical slice: one truck, one closed circuit of 16 Kenney tiles, a chase camera, three input devices, a finish line that counts laps, and a lap readout. About 800 lines with no comments.

The handling is arcade and hand written, not physics. The `Car` component carries a world `velocity`, a `yaw`, and a smoothed `steer`. Each frame the velocity splits into a part along the heading and a part sideways, and the sideways part bleeds off at a grip rate that drops when the handbrake is down. That is what makes a drift.

The chase camera follows the direction the car **travels**, not the direction it points, so a slide reads as a slide. The title screen looks down on the whole circuit.

`escape_key_exit_system` is deliberately **not** registered. Escape opens and closes the pause screen, and the title screen carries the Exit button.

The circuit is `SIDES = [8, 5, 8, 5]`, a ring one tile wide, so `queries::track_limits`
derives an outer and an inner rectangle from the tile centers and `drive` holds the car
between them. Any layout that is not a one tile wide rectangle breaks that derivation.

It runs everywhere: `Trunk.toml`, `index.html`, `X.toml` and `manifest.yaml` match the
other apps. On android alone, five pads and a pause button are drawn from `PAD_AREAS`,
fractions of the viewport that are both the rectangle that draws and the rectangle that
is touched. A touch that starts on a menu button counts as a press on every platform.

The tile facts the track layout depends on are in [[reference_kenney_racing_tile_edges]],
and the steering sign is in [[reference_heading_yaw_turns_left]].
