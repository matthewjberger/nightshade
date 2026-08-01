---
name: project_hearthside_app
description: apps/games/hearthside is the life sim on the Kenney furniture kit, where a placed piece feeds one need and the wage the household earns pays for the next piece
type: project
---

`apps/games/hearthside` is a small-house life sim. The lot is a 16 by 12 floor
on a green plot under a pan-orbit camera at a three-quarter view.

**The loop.** `data::CATALOG` holds six pieces, each naming the need it feeds,
its price, its fill rate, and its footprint radius. Placing a piece takes the
money and puts `Feeds`, `FillRate`, and `Footprint` on the prefab root. The
dweller reads her needs in order of urgency, takes the first one any placed
piece can serve, walks to a spot beside the nearest such piece, and stands
there while the need refills. Needs fall the rest of the time. The wage is
`WAGE_CONTENT` while every need is above `CONTENT_LINE` and `WAGE_STRAINED`
otherwise, so placed furniture is what pays for more furniture.

**Falling back past a need nothing can serve is deliberate.** An early lot has
one or two pieces, so the lowest need is usually one nothing feeds. Taking only
the lowest need would freeze the dweller in place with a full bed beside her.
`queries::needs_by_urgency` gives the order and `systems::dweller::decide`
walks it until a piece matches.

**The ghost.** One instance of every catalog prefab is spawned once, when the
models land, and parked at `systems::models::PARKED`, which is 80 units below
the floor. Aiming lifts the selected one to the cursor and leaves the rest
parked, so nothing is spawned or despawned while the cursor moves. A separate
plane under it carries a translucent material that is swapped between
`GHOST_CLEAR_MATERIAL` and `GHOST_BLOCKED_MATERIAL`, rather than being
recolored, because editing a registered material re-syncs every material that
frame. Only placed pieces carry `Footprint`, so a ghost never blocks itself.

**The buttons are held, not flowed.** `add_button` lays itself out as a flow
child of `Rl(100%, 0)`, so six buttons in one horizontal flow each take the
whole panel width and run off the screen. Each palette slot is its own
`window`-positioned holder with a vertical flow and one button inside it.

**The camera leaves the left button alone.** The pan-orbit bindings put orbit
on the middle button and pan on the right, so left click only ever places. The
pitch is positive, which is what puts the eye above the floor: refer to
[reference_pan_orbit_pitch_sign](reference_pan_orbit_pitch_sign.md).

**The shell** is `nightshade-shell` with `HearthsideShell` as the context and
four commands: `maxmoney`, `money`, `needs`, and `catalog`. A command reaches
the resource map directly and pushes a `HearthsideCommand`, which
`systems::needs::apply_commands` drains, so the shell names no system. Escape
closes the shell when it is open and closes the app when it is not.

The models are the Kenney furniture kit at `assets/furniture/`, CC0, read with
`include_bytes!` and loaded through `nightshade-gltf` as `ModelSource::Bytes`.
