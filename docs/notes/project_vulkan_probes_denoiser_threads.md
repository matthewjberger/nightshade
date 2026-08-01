---
name: project_vulkan_probes_denoiser_threads
description: The Vulkan backend carries a probe grid for bounced light, an SVGF denoiser, and threaded scope recording
type: project
---

`nightshade-vulkan` carries three things the wgpu backend does not:

**The probe grid (DDGI).** `renderer/scene/probes.rs` plus four compute
shaders: `probe_offsets`, `probe_trace`, `probe_irradiance`, `probe_distance`.
Each probe sends 64 rays, shaded by `scene_traced.glsl`, which
`traced_reflection.frag` shares, so the reflections and the probes shade a ray
hit the same way. Two atlases sized for a fixed maximum grid of 16x8x16, so the
descriptors never go stale: an irradiance atlas (8 texels per probe plus a
border ring) and a distance atlas (16 texels). `scene_probes.glsl` holds the
octahedral mapping and `probe_arriving_light`, which the scene fragment adds to
the sky term. Bound into scene set 0 at bindings 37 thru 42. Dispatched from
`ScenePass::fill_the_probes` in `offscreen`, beside `light_the_air`.

The relocation round matters: without it a probe inside a wall or inside a ball
sees only backfaces, carries no light, and takes every surface near it dark.
The visibility test must measure distance from the *biased* point and direction
from the *unbiased* one, or a dark seam appears where they disagree.

**The denoiser.** `Quiet` in `post.rs` plus `svgf_gather`, `svgf_variance`,
`svgf_wavelet`. It sits between the traced reflection and the existing screen
blur that lifts it to full size. The trap: the compose reads the reflection's
alpha as the confidence, so the confidence must be carried through all three
rounds. The variance rides a second target of its own.

**Threaded recording.** `renderer/record.rs`. A scope holding three passes or
more is recorded across up to four lanes, each with its own command pool for
each frame in flight. `Pass` requires `Send`, and the trait forbids a
pass to record a barrier in `draw`. The timing slots are
reserved on the handing-out thread and stamped on the recording one.

`apps/vulkan/traced_bounce` shows the probes: a Cornell box where space toggles
the grid.
