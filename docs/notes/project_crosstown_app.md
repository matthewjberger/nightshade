---
name: project_crosstown_app
description: apps/games/crosstown is the HD-2D beat-em-up, Kenney Toon Characters on a generated block street
type: project
---

`apps/games/crosstown` is a side-scrolling beat-em-up in the line of Streets of
Rage and Turtles in Time, drawn HD-2D: upright sprite fighters on a block city
street under a fixed tilted camera.

It carries two modes on app states. Story walks five stages (street, alley,
subway platform, rooftop, fenced yard) and ends on a boss. Arcade holds one
block and grows the wave each round. Four thug kinds sit in one table in
`data/tables.rs`: punk, runner, the brute that does not flinch, and Ironmouth.
Crates drop a pipe, a bottle, or food. Punching a thug who is already reeling
throws him, and the flung body knocks over what it lands on.

The art is Kenney's Toon Characters Pack 1, vendored under the app's own
`assets/`. Each character ships one
864x640 `Tilesheet/*_sheet.png` that is a 9 by 5 grid of 96x128 poses, and all
six characters share the same pose layout, so one index table serves them all.
The indexes that matter: idle 0, jump 1, fall 2, kick 14, attackKick 19,
attack0 27, attack1 28, attack2 29, hurt 33, walk0 thru walk7 36 thru 43,
fallDown 44. Two sheets pack into one 2048 atlas with a shadow blob.

The street carries no imported art. Its asphalt, paint, concrete, brick,
window, and metal textures are 16x16 pictures made in `commands/textures.rs`
the way `apps/games/farm` makes its grass and planks, and the whole block is
instanced cubes. That keeps the look cohesive and the repository small.

A jump raises `sprite.anchor.y`, not the transform, so the shadow blob stays on
the ground as a `ChildOf` child.

**Why:** the sheet layout and the generated-texture road are the two facts that
cost the most to rediscover.

**How to apply:** copy the pose table rather than parsing the Kenney XML again,
and reach for generated block textures before importing a tile pack.
A stage is swept by a `Piece(u32)` component holding the stage tag, so the hero
carries `Piece(u32::MAX)` and survives the sweep. A roof or platform needs a
floor that reaches its parapets and a solid block under it, or the camera sees
straight through the gap.

See [[reference_template_escape_exit_trap]].
