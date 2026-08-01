# nightshade-wgpu-sdf

Marches the `SdfFrame` that `nightshade-sdf` publishes.

The world bakes into a clipmap of sparse bricks and the march reads that atlas, so a field costs the bricks it occupies rather than its whole volume. The surface writes color, depth, the entity id, the view normal and the velocity the way the mesh pass does, reads the cascade shadow map and the image based lighting the meshes read, and draws its own depth into the cascade shadow map so meshes and the field shadow each other.

Compose `SdfRenderPlugin` beside the wgpu backend to turn it on.
