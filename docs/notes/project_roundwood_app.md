---
name: project_roundwood_app
description: apps/games/roundwood is the Animal Crossing style cylinder village
type: project
---

`apps/games/roundwood` is an Animal Crossing style village on a real cylinder.

The drum axis runs **left to right**, so the ground rolls away front to back and trees at the top of the screen lean back, which is the Animal Crossing look. The wrap direction is the walking-forward direction (`town.y`, `AROUND` tiles), and the bounded direction is across (`town.x`, `ACROSS` tiles, sea at both ends).

Two facts that the layout depends on:

- The world is a **turntable**: one parent entity at `(0, -RADIUS, 0)` holds every world entity as a child, and its rotation is the player's wrap angle. The camera never moves or rolls, which keeps the village on top of the drum where the sun reaches it and keeps the sky dome upright. Instanced meshes multiply by the parent transform on the GPU, so turning the world costs one uniform per instanced entity.
- `Atmosphere::DayNight` paints everything **below the horizon black**, and a cylinder always shows sky below the horizon past the drum's shoulder, so the whole frame goes black. `Atmosphere::CloudySky` keeps its horizon color below the horizon, which reads as sky. The app drives its own sun, hour, and weather because the engine's day/night system only runs under `Atmosphere::DayNight`.

The tiny_farm sheet has only two people (108, 109) and no fish or shell, so villagers, the shell, and the fish are **painted into the atlas from pixel tables in `data.rs`**. hd2d's person tile constants belong to its second sheet (tiny_dungeon) and are tools and furniture in tiny_farm.

Bands are the template's, with no comments anywhere.
