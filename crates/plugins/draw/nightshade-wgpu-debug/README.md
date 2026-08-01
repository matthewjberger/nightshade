# nightshade-wgpu-debug

Draws the lines and the ground grid a developer asks to see.

The lines pass takes the debug lines, the bounding volumes and the normals a
frame gathers, culls them on the GPU when the scene asks for it, and draws
them inside the scene overlay's shared render pass. The grid draws the ground
plane in the same pass.

Compose `DebugRenderPlugin` to draw them.
