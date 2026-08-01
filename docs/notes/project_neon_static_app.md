---
name: project_neon_static_app
description: apps/games/neon_static, the Hotline Miami style top-down shooter on nightshade-sprite
type: project
---

`apps/games/neon_static` is a top-down shooter of one-hit kills and fast
restarts.

Art is the Kenney Top-down Shooter pack, copied to
`apps/games/neon_static/assets/kenney_top_down_shooter/`. Two files carry it
all: `tilesheet_complete.png` is an exact 27 by 20 grid of 64 pixel tiles, and
`spritesheet_characters.png` is 512 by 256 with named rectangles in the XML
beside it. Both go into one 2048 sprite atlas. Tile numbers that matter: 58 is
plain dark asphalt, 12 and 13 are orange wall blocks, 11 is white, 185 is a
filled blob that takes a red tint as blood, 290 is a small shard for shell
casings. The black splat at 319 stays black under any tint, so it is useless
for blood.

Every character rectangle is 43 tall and the body center sits 15 pixels from
the left edge, so the sprite anchor is `15.0 / pose_width` and the art faces
+X at angle zero.

It is the live example of a component that holds one value: the components are
`Angle`, `Speed`, `Cooldown`, `Velocity`, `Spin`, and `Life`, and the queries
pick the kind out by composition.
