# nightshade-wgpu-sprites

Draws the `SpriteFrame` that `nightshade-sprite` publishes.

The pass installs over the lit meshes and reads the scene colour and depth by name. It owns the sprite atlases, uploads the pictures the frame carries, and draws every instance sorted by layer, then depth, then Y, tested against the scene depth without writing it.

Compose `SpriteRenderPlugin` beside a backend to turn sprites on.
