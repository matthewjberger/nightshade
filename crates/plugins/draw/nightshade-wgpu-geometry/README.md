# nightshade-wgpu-geometry

Everything that draws the scene itself: the static meshes, the skinned meshes, the shadow maps, the sky, and the overlay that carries the grid, the debug lines and the world text.

The three drawing passes share one bindless material binding, one cull buffer and one shading vocabulary, so they are one capability rather than three. The crate owns the material texture arrays and the uploads that fill them.

Compose `GeometryRenderPlugin` beside a backend to draw a scene.
