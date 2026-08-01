# nightshade-wgpu-sky

Draws the sky behind the scene and works the image based lighting out of it.

The pass draws a procedural atmosphere or a loaded high dynamic range picture,
captures that into a cubemap, filters it into an irradiance map and a
prefiltered map, and publishes both into the graph for the lit passes to read.
Over a day and night cycle it holds a bracket of snapshots and blends between
the two the hour falls between.

Compose `SkyRenderPlugin` to draw it.
