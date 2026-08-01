# nightshade-wgpu-particles

Draws the `ParticleFrame` that `nightshade-particles` publishes.

The emitters update on the GPU and the particles draw inside the scene overlay's shared render pass, so they cost no second attachment round trip.

Compose `ParticleRenderPlugin` beside a backend to turn it on.
