# nightshade-wgpu-outline

Draws an edge around every entity the scene marks with `Outlined`.

The mask pass reads the entity identifier target and writes a one channel
mask for the marked entities. The edge pass widens that mask and blends the
result over the scene colour.

Compose `OutlineRenderPlugin` to draw it.
