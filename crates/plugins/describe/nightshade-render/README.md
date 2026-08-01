# nightshade-render

The render description is the data both GPU backends read. This crate holds the per-entity render components, the materials, meshes, textures and lights, the render config, and the registries and caches that hold them. It also carries the CPU math that is the same whichever backend draws: the reverse-Z projections and cascade tests in `queries::projection`, `queries::lights::build_render_lighting`, and the image decoding in `data::decoded_image`. It names no GPU API and carries no system and no plugin.

## Composing it

The crate carries no plugin. `nightshade-graphics` installs the resources declared here, and a program composes a backend's `DefaultPlugins`, which pulls graphics in.

## Using it

```rust
commands.spawn((
    Name(name),
    LocalTransform {
        translation: surface.at,
        rotation: surface.rotation,
        scale: Vec3::new(1.0, 1.0, 1.0),
    },
    GlobalTransform::default(),
    Light {
        light_type: LightType::Area,
        color: tint,
        intensity,
        range,
        area_shape: surface.shape,
        area_width: surface.width,
        area_height: surface.height,
        area_radius: surface.radius,
        cast_shadows: false,
        ..Default::default()
    },
));

let settings = resources.res_mut::<RenderSettings>();
settings.atmosphere = Atmosphere::None;
settings.anti_aliasing = AntiAliasing::Taa;
```

A `Light` on an entity that already carries a transform becomes a light the frame draws with, and `LightType` picks which kind. `RenderSettings` is the resource holding the frame's look, the atmosphere, the tonemap, the color grading, the fog and the anti-aliasing among them. `MaterialRegistry`, `MeshCache` and `TextureCache` are the registries a program reaches for when it names a material, a mesh or a texture, and `data::material::Material` is the value a material holds.

## Taking it from another crate

The crate does not split at a `contract` feature, because the whole crate is already the contract. A crate that reads the render description takes it whole, and `contract.rs` gathers the camera inputs and the light types a plugin most often names.
