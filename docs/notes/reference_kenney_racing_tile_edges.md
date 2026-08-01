---
name: reference_kenney_racing_tile_edges
description: Kenney racing kit tile geometry - 10x10 tiles, straight road along local Z, corner joins +Z to -X
type: reference
---

The Kenney racing starter kit has tiles that are exactly 10 x 10 units, x and z both -5 to 5. Every model references `Textures/colormap.png` as an external file, so it loads with `ModelSource::Parts` and that key.

Where each road meets the tile edge:

- `track-straight` and `track-finish`: `z = -5` and `z = +5`. The road runs along local **Z**.
- `track-corner`: `z = +5` and `x = -5`. The corner joins **+Z** to **-X**.

So a corner placed on a grid takes the yaw of the direction the car **leaves** in, not the direction it arrives in. Rotating by the arrival direction puts both road edges on the far side of the cell and the incoming straight ends at grass.

The truck models are about 1.5 wide by 2.8 long with wheels of radius 0.3, so they need no scaling against a 10 unit road.

To get facts like these out of a `.glb` without guessing, read the JSON chunk at byte 20 for `jsonLength` bytes, then walk the POSITION accessors through their bufferViews into the BIN chunk. See [[project_hairpin_app]].
