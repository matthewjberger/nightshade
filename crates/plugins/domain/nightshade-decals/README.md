# nightshade-decals

Decals are pictures projected onto whatever geometry stands under them, so a scorch mark or a stain follows the surface it lands on. A `Decal` on an entity that already carries a transform is gathered every frame into `DecalFrame`, which the backend reads and draws. The crate names no GPU API and owns no pass of its own.

## Composing it

```rust
app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
    .add_plugin(nightshade_text::TextPlugin)
    .add_plugin(nightshade_decals::DecalsPlugin)
    .add_plugin(nightshade_gltf::GltfPlugin)
    .run()
```

`DecalsPlugin` inserts the `DecalFrame` resource and registers `systems::gather_decals_system` on `Stage::Render`.

## Using it

```rust
commands.spawn_reserved(
    entity,
    (
        Name(format!("Decal_{}", entity.id)),
        LocalTransform {
            translation: position,
            rotation: rotation_from_normal(normal),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
        Decal::new(texture)
            .with_size(size, size)
            .with_color(color)
            .with_emissive_strength(emissive)
            .with_depth(size),
    ),
);
```

`Decal::new` takes the name of a texture, and the builder calls set the projector's footprint, tint, emissive strength, projection depth, normal threshold and distance fade. The transform places and orients the projector, and the gather system reads `GlobalTransform` and the optional `Visibility` off the same entity. Each row becomes a `DecalInstance` in `DecalFrame`, carrying the transform and the visibility flag beside the decal's own values.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-decals = { path = "../nightshade-decals", default-features = false, features = ["contract"] }
```

The contract carries `Decal`, `DecalFrame` and `render_data::DecalInstance`, which is what a backend needs to draw the frame without linking the gather system.
