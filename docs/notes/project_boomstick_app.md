---
name: project_boomstick_app
description: apps/games/boomstick is the boomer shooter, six hand-authored foundry floors with keyed doors, secrets, four guns and a rocket jump
type: project
---

`apps/games/boomstick` is the reference for a first person shooter in this
engine.

- `data/levels.rs` holds six `LevelPlan` entries. A plan names rooms, the gaps
  cut in their walls, ledges, pillars, crates, lamps, keyed gates, shootable
  levers, breakable secret walls, hazard volumes, ambush volumes, monsters,
  loot and the exit. `systems/level.rs` turns a plan into geometry.
- `data/arsenal.rs` holds the four guns and `data/loot.rs` holds the pickups,
  each one a list of cube `Part` entries from `data/shape.rs` that
  `commands::build_parts` spawns. The same parts make the viewmodel and the
  pickup on the floor, so the monsters are the only models the app imports.
- `data/bestiary.rs` holds the five monsters. A monster wakes when it sees the
  player through a raycast, when the player crosses its ambush volume, or when
  it is shot, and one that carries a `keep` range holds that range and fires.
- `contract.rs` carries the fire queue. The weapon writes shots into it, the
  enemies write bolts, a bolt that lands writes a blast, and `systems/shot.rs`,
  `systems/projectiles.rs` and `systems/blast.rs` read them in that order.
- Screens are a `Screen` enum driven by `insert_state` with panels from
  `nightshade-ui-menu`: Title, Level, Paused, Tally and Ending. Escape pauses and
  a button quits, so the app registers no escape exit system.
- `--level <n>` opens a floor directly, because a capture cannot press a key.
- `systems/pace.rs` sets the physics step to the frame's own duration, and the
  player body carries `PhysicsInterpolation` disabled. A camera parented to a
  body inherits whatever the body's step is, and at this run speed a sixty
  hertz step shakes the view.

Two shapes of geometry bite here. Two rooms that share a wall line both build
a wall on it, and a lintel sits exactly on the door below it, which puts two
faces in one plane and makes them fight. `systems/level.rs` claims each wall
line for the taller room and gives every piece a small bite into its
neighbour. A point light close to a flat ceiling makes a bright ring that
bloom turns into a rainbow, so every light is clamped to half the room height.

The monsters are skinned, so the height they draw at comes from the skin rather
than from the vertex data the file carries. `data::MODEL_HEIGHT` holds that
height, and `systems/populate.rs` divides the height a monster wants by it.
