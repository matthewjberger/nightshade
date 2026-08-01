# nightshade-wgpu-shading

The vocabulary every lit pass over the wgpu backend speaks.

It publishes the nine shader modules those passes import, from the physically
based reflectance math to the material sampling, the fog and the shadow
cutout. Beside them it holds the shared Rust the same passes read: the light
cluster grid, the linearly transformed cosine tables for area lights, the
material bind group builders, the scene uniforms, the shadow projections and
the shadow buffers.

Compose `ShadingPlugin` before any plugin that draws a lit pass.
