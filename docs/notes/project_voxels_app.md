---
name: project_voxels_app
description: apps/games/voxels, the block world with a main menu and a world plan the player sets before the world is built
type: project
---

`apps/games/voxels` is the block game: chunk streaming over columns and slabs,
greedy meshed, with biomes, villages, dungeons, caves, weather, farming, boats,
chests, monsters and a day and night cycle. The world is on the disk at
`nightshade-voxels/world.save` through `nightshade-store`.

It opens on a title screen (`Screen::Title`) over the world,
with `systems::world::look_over_the_world` turning the camera, and a new world
screen (`Screen::NewWorld`) where the player sets a `WorldPlan`: the shape of
the land (Rolling, Flat, Islands, Peaks), the seed, how many trees, caves,
villages, dungeons, monsters, weather, and the hour the world starts at.

- The plan rides on `Generation`, so each generation pass reads it.
- `WorldPlan::default` is seed 1337 with every multiplier at 1.0, so the
  default plan and a given seed always give the same blocks.
- The save is version 2 and carries the plan. A version 1 save still opens and
  takes the default plan.
- A new world is built in two steps: `start_the_world` queues the teardown of
  the world on screen, and `raise_the_new_world` builds the ground in the frame
  after, because those entities go out with the command queue.

Related: [[reference_app_material_must_be_held]]
