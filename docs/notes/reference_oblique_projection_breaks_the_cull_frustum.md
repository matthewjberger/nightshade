# An oblique projection needs its own frustum for the cull

A camera that carries a `ProjectionOverride` for a portal or a mirror holds an
oblique projection: row two is replaced by the plane of the opening, so
nothing in front of that plane draws. Rows zero, one and three are untouched.

`extract_frustum_planes` builds the near and the far plane out of row two, so
an oblique matrix gives a near plane that is the opening, which is wanted, and
a far plane that points at nothing the camera can see, which is not. Culling
against it drops geometry that is in view. The symptom is a pillar or a wall
that vanishes and comes back as the viewer turns a little, inside the view
through the opening and never outside it.

So the cull uses its own matrix. `CameraMatricesInputs` carries
`cull_projection` beside `projection`. `nightshade-camera` fills it with the
plain reverse-Z perspective built from the override's own `y_fov_rad`,
`aspect`, `z_near` and `z_far`, and with the projection itself for every
ordinary camera. `build_render_view` extracts `RenderView::frustum_planes`
from that matrix, and the GPU cull in the mesh pass reads those planes rather
than extracting its own.

The plain perspective is wider than the oblique one on every side, so the cull
keeps a little more than the frame needs and never drops what it can see. A
cull must lean that way.

The occlusion test cannot be repaired the same way, so an oblique camera does
not run it. It projects the corners of a bound with the camera's own matrix
and compares the depth against a hierarchical depth buffer, and the mesh pass
holds one of those for every camera it draws in a frame. An oblique depth is
not comparable with the depth any other camera wrote, so the test reads a
value that means nothing and hides what is in view. `RenderView` carries
`oblique_depth` for that, and the cull uniforms turn occlusion off when it is
set. Frustum culling still runs.
