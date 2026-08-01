# Surfaces

Two numbers decide most of what a surface looks like. Roughness says how
broadly it scatters, and metallic says whether the colour you gave it tints
the reflection or sits under it.

```demo
id: material_sampler
title: Roughness and metal
caption: Five spheres, each naming its own material. Drive the sliders and every one changes, because the demo writes through the registry rather than through a copy.
```

## Roughness

Roughness runs from 0 to 1.

- At 0 the surface reflects in a sharp, mirror-like lobe
- At 1 it scatters over the whole hemisphere and reads as chalk
- Most real materials sit between 0.3 and 0.7

A value of exactly 0 is a poor default. A perfectly smooth surface reflects
only what is actually in the scene, so a lone sphere with no environment
around it reads as black.

## Metallic

Metallic is close to a switch rather than a dial.

| Value | Behaviour |
| --- | --- |
| 0.0 | Dielectric. White specular, `base_color` shows as diffuse |
| 1.0 | Conductor. `base_color` tints the specular, no diffuse at all |

Values in between describe no real material. They exist so a texture can
blend across a boundary between one and the other.

> A metal with no light and no environment to reflect has nothing to show. If
> a metallic surface reads as flat black, the material is right and the
> lighting is missing.

## Emission

`emissive_factor` is light the surface makes on its own. It is not clamped to
1, and values above 1 are what make a surface bloom:

```rust
Material {
    base_color: [1.0, 0.95, 0.85, 1.0],
    emissive_factor: [3.0, 2.8, 2.4],
    roughness: 1.0,
    ..Default::default()
}
```

An emissive surface lights itself and nothing else. To light the scene, put a
`Light` on the same entity.

## Sharing one material

The registry holds one copy per name, so mutating it moves every entity that
names it:

```rust
material_registry_mutate(registry, "ball", |material| {
    material.roughness = wanted;
    material.metallic = metal;
});
```

The figure above uses that directly. Five spheres, five names, and one write
per frame per name.

## Transmission and volume

Glass needs three fields together, not one:

```rust
Material {
    base_color: [1.0, 1.0, 1.0, 1.0],
    roughness: 0.05,
    transmission_factor: 1.0,
    thickness: 0.5,
    ior: 1.5,
    ..Default::default()
}
```

A transmission factor without a thickness reads as a thin film. Thickness is
what bends the light on the way through and gives glass its weight, and `ior`
says how far it bends.

## Thin films

A film over a surface splits the light that reflects off it, so the color you
see turns with the angle you see it from. The spheres below carry one material
each, differing only in how rough they are.

![Seven spheres under a thin film, from rough on the left to smooth on the
right](images/iridescence.jpg)
