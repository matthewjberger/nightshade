# cobble

A bare voxel world. The terrain, the biomes and the villages come up out of a seed, and you walk in them, dig them and build on them. There is no head-up display, no villager, no beast, no quest, no chat and no save. A crop that you plant stays as it is, because the farm crate is not composed.

It is the smallest thing the blocker crates make. The app names seven of them:

```
blocker-screen    the screen and the notice
blocker-blocks    the blocks, the tools, the items, the atlas and the meshes
blocker-figure    what a body has left and what takes it away
blocker-pocket    the hotbar and the chests
blocker-weather   the sky, the season and the rain
blocker-world     the terrain, the biomes and the villages
blocker-player    the body, what it aims at and what it holds
```

What the app itself holds is what no crate can hold for it: the composition, the render settings, the sun, the camera and the body at startup, and the seed the draft starts from. The body carries no model here, so nothing walks and nothing is named, but it is still the entity the player stands at.

## Running it

```
just run cobble
```

Click to take the cursor. The blocker actions apply, so the movement keys walk, the mouse digs and places, and the number keys pick from the hotbar.
