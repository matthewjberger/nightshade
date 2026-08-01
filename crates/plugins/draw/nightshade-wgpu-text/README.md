# nightshade-wgpu-text

Draws the text a scene places in the world or on the screen.

The pass takes the glyph meshes the text crate builds, samples the glyph
atlas the graph carries, and draws inside the scene overlay's shared render
pass so the text needs no attachment of its own.

Compose `TextRenderPlugin` to draw it.
