---
name: reference_volumetric_fog_draws_sun_shafts
description: Volumetric fog lights a grid against the sun cascades, so every caster throws a shaft across the view, and a density of zero is not off
type: reference
---

`Fog { volumetric: true }` lights a grid of 160 by 90 by 64 cells over the
view and asks the sun's cascades whether each cell is in shadow, in
`fog_scatter.wgsl`. Every caster in the world therefore throws a shaft through
the air, and a surface reads what the air in front of it added. In a small
scene the shafts read as god rays. In a large one, with a wood of trees and a
fog that reaches hundreds of units, they read as long straight bands that lie
across the whole view, cross each other, and do not follow the ground. That
picture looks like broken shadows, and the shadow map is not the cause.

A density of zero does not turn the air off. The shader reads

    if density <= 0.0 { density = 3.0 / (fog_end - fog_start) }

so a fog that reaches 340 units still carries a real density. Only the
`volumetric` flag decides whether a surface reads the lit air at all, in
`mesh.wgsl`.

A zone that wants distance fog and no shafts sets `volumetric: false` and
keeps the color, the start and the end. It gives up the lit air the lanterns
glow through at night, which comes from the same grid.

The other half of a large zone is the shadow bias. `spawn_sun` hands out
0.0005, which the directional cascade path uses whenever it is above zero.
Over cascades that reach 150 units a texel covers a large piece of ground, and
that bias is far under what the slope needs, so the flat surfaces stripe with
acne. The apps that draw large scenes set 0.02 with a normal bias near two.
