---
name: reference_cluster_assign_bind_group_follows_the_lights_buffer
description: The cluster light assignment binds the mesh lights buffer, so a lights buffer that grows past its capacity must drop that bind group too, or every local light past the thirty-second reads a buffer nothing writes
type: reference
---

The mesh pass keeps its lights in one storage buffer that starts at
`INITIAL_LIGHTS` slots and grows when a frame collects more. Two bind groups
bind that buffer: the instance bind group the mesh shaders read through, and
the cluster assign bind group the `cluster_light_assign.wgsl` compute pass
reads to decide which local lights each cluster holds. When the buffer grows,
the growth code rebuilds the instance bind group at once and creates the
cluster assign bind group again only when it is `None`.

A growth that rebuilt the first and kept the second left the cluster pass
bound to the old buffer, which the frame never writes again. The old buffer
held the lights of the frame before the growth, the count in the cluster
uniforms was the new one, and the compute pass placed each light in the
clusters of whatever the old buffer held at that index. The fragment shader
then read the correct light through the rebuilt instance bind group but only
in the clusters the stale positions chose, so a point light past the thirty-
second lit nothing near it. A world with few lights never grows the buffer
and never shows it, which is why the voxels game with a village of torches
lost its torch light and every other app kept its lights.

The rule is the one the skinned mesh pass already keeps: a resize of the
lights buffer sets `cluster_assign_bind_group` to `None` in the same step,
and the growth runs before the block that creates the bind group, so the
frame that grows also binds the new buffer and no frame draws without its
local lights. Both mesh resize sites, `prepare_pass_node` and
`prepare_uniforms_and_lights`, do this.
