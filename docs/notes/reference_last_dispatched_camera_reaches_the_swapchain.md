# The camera that renders last is the one the swapchain shows

`frame.rs` loops over the dispatched cameras and points the render graph's
viewport resource at each camera's texture in turn. The swapchain passes run
once, after the loop. So the surface carries the texture of the camera the
loop reached **last**, whatever `ActiveCamera` names.
`is_main_viewport_camera` only decides the size a camera renders at.

The order comes from `Viewport::required_cameras`, and then from
`publish_render_views`, which appends every camera that carries a
`RenderTarget` and is not already in that list. A camera left out of the list
therefore lands at the end and takes the screen.

So an app or a plugin that adds an offscreen camera, a portal eye, a mirror or
a monitor, must keep the viewer at the end of the list.
`nightshade-portal` inserts each new camera at the front for that reason, and
that also puts the deepest level of a portal chain first, which is the order
the chain needs.

The symptom is a full screen view from the offscreen camera, which reads as
the player standing outside the level. Turn off the pass that draws the
offscreen image to tell the two apart. If the wrong view stays, the cause is
the camera order and not the pass.
