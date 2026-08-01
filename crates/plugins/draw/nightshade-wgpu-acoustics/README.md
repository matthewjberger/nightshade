# nightshade-wgpu-acoustics

Works out what a listener hears from the grid that `nightshade-acoustics` gathers.

The grid runs as a compute pass off a pre-render hook with no graph node, and the readback reaches the frame after.

Compose `AcousticsRenderPlugin` beside a backend to turn it on.
