# nightshade-wgpu-near

Answers the proximity asks that `nightshade-near` gathers.

One compute pass clears a bucket grid, fills it by atomic append, answers every ask and works out the push for every crowded body, then copies the answers back without blocking. The answers belong to the frame before, the way a GPU pick comes back after the click.

Compose `NearRenderPlugin` beside a backend to turn it on.
