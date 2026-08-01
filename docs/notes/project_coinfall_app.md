---
name: project_coinfall_app
description: apps/games/coinfall is the Kenney pixel platformer whose clock always falls and whose coins buy it back
type: project
---

`apps/games/coinfall` is a 2D pixel platformer. The clock always falls. A coin gives 2 seconds back, a stomped walker gives 3
coins, a side hit costs 3 seconds, and the flag banks what is left as the
score. Five levels of rising difficulty, from First Light at 34 seconds to Last
Call at 26, with an overworld of dots between the title and the level.

The control is hand written against a tile grid in `systems/player.rs`, not a
physics engine, because an exact platformer needs it: axis separated AABB
resolution, a jump whose height follows the button, stronger gravity on the way
down, coyote time, and a jump buffer. `nightshade-physics` is held back
permanently for this reason.

Levels are the `LEVELS` table in `data.rs`, each a `LevelSpec` whose `rows` are
text. `GROUND`, `BLOCK`, `COIN`, `WALKER`, `START`, and `FLAG` are the
characters, so count a row's length against its neighbors and keep a prop off
a hole when you edit one. Each level carries a `Palette`:
a terrain set, two columns of the backgrounds sheet, a walker character, and a
tilemap tint. `systems/level.rs` despawns every `LevelPiece` and rebuilds when
`wanted_level` is set.

**Kenney Pixel Platformer tile facts.** Terrain sets run `row*20 + 0..3` as
single, left, middle, right, and only rows 0 to 5 are terrain runs: tiles 120
to 123 are standalone bordered blocks and draw as broken bricks if used as
ground. Fill tiles are 4, 5, 24, 25. Crate 9, flag 111, coin 151. Props: pine
126, cactus 127, bush 125, plant 124, mushroom 128, fence 105, bones 144,
snowman 145. Characters are 9 by 3 of 24 texels: 0 and 1 the green player, and
11 to 25 the usable enemies. Avoid 13 and 14, a gold badge that reads as a
coin. The backgrounds sheet is 8 by 3 of 24 texels in three palettes, pale 0 to
3, orange 4 and 5, green 6 and 7, with sky, horizon, and base stacked by row.

**Two traps.** A coin hung in a pit below the platform tops is unreachable, and
reaching for it is a death, so place coins on the line the jump actually takes
and test collection against the player's whole body rather than a point. A
retained UI node paints nothing without `with_rect`, so a full screen veil needs
one even when its border is clear.

**Why:** the whole design hangs on one rule, that the clock is the only failure
channel, so every mechanic either spends time or buys it.

**How to apply:** grow it with checkpoints, springs, breakable blocks, and a
time attack mode before adding a second failure channel.
