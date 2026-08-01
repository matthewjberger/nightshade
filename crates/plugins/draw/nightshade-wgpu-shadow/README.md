# nightshade-wgpu-shadow

Draws the depth every shadow is read from.

One pass fills the cascade atlas for the sun, the atlas slot of each
spotlight and the cube face of each point light, culling the occluders on the
GPU and drawing the skinned ones from the poses the frame wrote.

Compose `ShadowRenderPlugin` to draw it.
