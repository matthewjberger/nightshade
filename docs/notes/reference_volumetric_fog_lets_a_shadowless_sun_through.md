---
name: reference_volumetric_fog_lets_a_shadowless_sun_through
description: The volumetric fog pass occlusion-tests only the first directional light, and skips that test when shadows are off, so a fill light washes a sealed interior
type: reference
---

`fog_scatter.wgsl` gathers the air's in-scattering from every light in the
cluster. The spot and point arms each call `spot_reaches` and `point_reaches`,
so a local light is tested against its own shadow map before it lights a cell.
The directional arm is not the same:

    for (var index = 0u; index < cluster_uniforms.num_directional_lights; ...) {
        var reaching = 1.0;
        if index == 0u {
            reaching = sun_reaches(world_position, depth);
        }
        gathered = gathered + light.color.rgb * reaching * phase(...);
    }

Only the light at index zero is tested. Every directional light after it adds
its whole colour to every cell of the volume, with no occlusion of any kind.
And `sun_reaches` itself opens with

    if uniforms.shadows_enabled < 0.5 { return 1.0; }

so even the first one is untested when the sun casts no shadows.

The result is that a dim shadowless directional light, composed as a fill so an
indoor scene is not pure black, lights the air inside a sealed room as though
the room had no roof. The picture reads as a uniform haze in the light's own
color that walls do not stop, and turning the light's intensity down thins it
rather than removing it, because the haze is in every cell rather than in a
beam. `nightshade-render` folds intensity into color in `queries/lights.rs`, so
intensity is the only handle the fog gives you.

An indoor scene that wants volumetric fog therefore wants no fill light at all.
Light it with the spot and point lights it already carries, which the fog does
occlude, and let the unlit parts be black. `apps/games/blackmarrow` is the live
example.
