# Lighting

A light is a component on an entity. It has no separate list, no manager, and
no registration. Where the entity is, the light is.

```demo
id: light_rig
title: One light around a sphere
caption: A point light on a ring, casting a shadow onto the floor. Raise it, brighten it, and stop it where you want to look.
```

## Putting one in the world

```rust
world.spawn((
    Light {
        light_type: LightType::Point,
        color: Vec3::new(1.0, 0.94, 0.86),
        intensity: 14.0,
        range: 24.0,
        cast_shadows: true,
        ..Default::default()
    },
    LocalTransform {
        translation: Vec3::new(2.2, 1.6, 0.0),
        ..Default::default()
    },
));
```

Moving the entity moves the light. The shadow follows on the same frame,
because both read the same transform.

## The three kinds

| Type | Position | Direction | Range |
| --- | --- | --- | --- |
| `Directional` | Ignored | Used | Infinite |
| `Point` | Used | Ignored | `range` |
| `Spot` | Used | Used | `range`, inside the cone |

A directional light is the sun. One of them lights the whole world at a
constant angle, and its shadow is cascaded so that near geometry gets the
resolution and far geometry gets the coverage.

## Range and falloff

`range` is where a point or spot light stops contributing. It is a cutoff for
culling rather than a physical quantity, so set it to where the light is no
longer visible and not further. A range far larger than the lit area costs
shadow resolution and gains nothing.

## Shadows

`cast_shadows` is per light, and it is not free:

- A directional light renders one pass per cascade
- A point light renders six faces
- A spot light renders one

Turn it on for the lights that matter and leave it off for fill. A scene with
one shadowing key light and four unshadowed fills looks better and costs less
than five shadowing lights.

## Ambient and the sky

An entity lit by direct light alone has black shadows. The fill comes from the
sky, and the sky is one setting:

```rust
let settings = resources.res_mut::<RenderSettings>();
settings.atmosphere = Atmosphere::Sky;
```

The variants run from `None` through `Sky`, `CloudySky`, `Sunset`, `Space`,
and `Nebula` to `DayNight` and the two that read a loaded HDR.
`Atmosphere::None` gives a flat backdrop and leaves the lights to do the whole
job. The figure above uses that, so what you see is one light and nothing
else.

## Exposure

The renderer tone maps, so intensity is not brightness on screen. Doubling a
light's intensity in a scene with automatic exposure moves the balance between
lit and unlit, and moves the final image far less than the number suggests.

![The same hall from further down it, where the light falls off and the fog
carries what is left](images/sponza-nave.jpg)
