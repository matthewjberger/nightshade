# nightshade-portal holds the portal capability, and the backend draws it

A portal is an entity that carries three components from
`crates/plugins/domain/nightshade-portal`: `Portal` with the plane it sits on, `Link`
naming the opening it looks out of, and `Rim` with the color of its edge. The
app spawns those and nothing else. `PortalPlugin` does the rest.

`systems::view::open` gives every portal without cameras a chain of them, one
for each level of depth, and records the chain as `Eyes`.
`systems::view::aim` points each camera through the partner opening, writes
the oblique projection that clips at the opening so nothing in front of it
draws, sizes the render target to the viewport, and publishes a `PortalFrame`.
A backend reads that frame and draws each opening with the texture of the
camera at its level.

The frame is the contract, so the crate names no GPU API.
`nightshade-wgpu` owns `passes/portals.rs` and `shaders/portal.wgsl` and reads
the frame in a pre-render hook, the same shape as decals and sprites. The
Vulkan backend draws no portal, because it renders one camera to the
swapchain and has no offscreen camera targets at all.

`PortalSettings` carries the camera the openings are seen from, the projection
numbers, the size of each target, how many openings deep the view goes, and
the distance past which an opening is not drawn. The depth is read when an
opening gets its cameras. An opening the viewer stands behind, or one farther
away than the limit, has its cameras set to `ViewportUpdateMode::Disabled`, so
the scene is not rendered again for it.

The crate reads the viewer's `LocalTransform`, or its `GlobalTransform` when
it is a child of something else. `LocalTransform` is what the app wrote this
frame, so the view through an opening does not lag the viewer by a frame.

Two apps use it and no app carries a portal of its own.
`apps/games/portals` is a room of two halves with an opening in each, and a
gun that moves them. `apps/features/hall_of_mirrors` is a corridor with an
opening at each end that face each other, so the view runs down a tunnel of
five levels. Each app keeps its room, its player and its interface, and
nothing else.
