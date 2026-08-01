---
name: project_bulwark_app
description: apps/games/bulwark is the isometric tower defense on the Kenney tower defense pack, where a four bit neighbour mask picks the road tile
type: project
---

`apps/games/bulwark` is a tower defense on seven hand laid boards. Crystals walk
a road to a keep, towers stand on the grass beside it, and the whole thing
draws through `nightshade-sprite` under a flat orthographic camera. The
isometric look is the art and a diamond grid the app lays out itself.

Art is the Kenney Tower Defense (isometric) pack, cut down to
`apps/games/bulwark/assets/kenney_tower_defense/`. Its geometry is worth
knowing before you reach for the pack again.

Every landscape tile carries its own ground block, and the centre of that
block's top face is 66 texels above the image bottom whatever the tile's
height: 99 for plain grass, 112 for a crystal tile, 130 for a tall oak. So a
sprite anchors at `(0.5, 66.0 / height)` and its top face lands on the grid
point. That also means a scenery tile replaces the ground tile rather than
standing on top of one. Stack them and you get a step and a seam at every
tree.

Only `landscape_13` is a flat plain green tile. The other green tiles are
ramps and shaded slopes, so a field built from a mix of them breaks into
ledges. Vary the field with a per cell tint on the one flat tile instead.

The road tiles meet at the four midpoints of the diamond's edges, not at its
corners, so a road tile is picked by a four bit mask of which neighbours are
road. With `+column` drawn toward the lower right and `+row` toward the lower
left, the bits are northeast for `row - 1`, southeast for `column + 1`,
southwest for `row + 1` and northwest for `column - 1`. The eleven tiles that
answer every mask are landscape 29, 32, 02, 03, 07, 31, 04, 09, 06, 10 and 30,
and they are copied in named for the mask they serve. An end of the road has
one neighbour, so it takes the straight that contains that side and the road
runs off the board edge, which is what you want at a spawn and a keep.

The tower sheets are modular blocks, so a block sprite anchors by its own
geometry rather than a fixed number: find the lowest painted row, take the
widest span in the bottom half as the base diamond's width, and the base
centre sits a quarter of that width above the lowest row. `commands/art.rs`
works it out at pack time, which is why the keep and nine tower sprites of
four different sizes all sit flush on their tiles.

The pack carries no creep, so the crystals are cut out of the crystal tiles by
colour: keep the purple, take the bounding box, then keep every pixel inside
it whose blue beats its green. That leaves the shard without the grass it
stood on.

The pack draws crystals as scenery and the game walks crystals down the road,
so a field scattered with them reads as a field full of enemies. Keep the
crystal scatter to one cluster on a board and let rocks and trees carry the
rest, and the only purple thing that moves is the one worth shooting.

A sprite that carries state in its color needs that color kept somewhere else
as well. The hit flash, the chill and the ward each write `Sprite::color`, and
whichever of them ends last has to put back the color the crystal was spawned
with rather than plain white, so the crystal holds a `Tint` component and every
one of those passes reads it.

One engine trap cost a real bug here. `menu_show_on` registers a panel against
a screen, and registering the same panel for two screens shows it on neither,
because the pass that hides the panels of every other screen hides it again.
Defeat and victory each need their own panel, heading and line.
