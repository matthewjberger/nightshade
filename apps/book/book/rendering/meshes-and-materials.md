# Meshes and materials

An entity draws by naming a mesh and a material. It holds neither. The mesh
lives in a cache under a name, the material lives in a registry under a name,
and the entity carries the two names and a transform.

```demo
id: orbiting_cubes
title: Cubes on an orbit
caption: Each cube is one row: a mesh name, a material name, and a transform. Add and remove them while the scene runs.
```

## The mesh cache

A mesh goes into the cache under a name, and the name is what an entity holds:

```rust
let cache = resources.res_mut::<MeshCache>();
mesh_cache_insert(cache, "cube".to_string(), create_cube_mesh());

commands.spawn((
    RenderMesh::new("cube"),
    MaterialRef::default(),
    LocalTransform::default(),
));
```

Ten thousand entities naming `"cube"` share one upload. The renderer batches
them into a single instanced draw without being told to.

## Reclamation

A system in `Stage::Last` reclaims what nothing names. That is what you want
for a level that unloads, and what you do not want for a mesh you are about to
spawn against again:

```rust
mesh_cache_protect(cache, "cube".to_string());
material_registry_release(registry, "stone");
```

Both take the name and set a flag on the slot behind it. `insert_material`
protects what it registers, so a material an app owns is already held.

Protect a mesh for as long as your scene owns it, and release the protection
when you tear the scene down. The figure above does exactly that, which is why
removing every cube and adding one back still works.

## Bounds

The renderer culls against a bounding volume, and a mesh built by hand carries
none. A mesh with no volume is culled against a shape it never drew, which
reads on screen as geometry that is simply not there.

```rust
let bounds = BoundingVolume::new(
    OrientedBoundingBox::from_aabb(lowest, highest),
    (highest - lowest).norm() * 0.5,
);
let mesh = mesh.with_bounding_volume(bounds);
```

Give the mesh its volume before it reaches the cache. Nothing later in the
frame will work one out for you.

## Materials

A material is registered once and referenced by name:

```rust
let registry = resources.res_mut::<MaterialRegistry>();
material_registry_insert(
    registry,
    "brass".to_string(),
    Material {
        base_color: [0.85, 0.68, 0.30, 1.0],
        roughness: 0.35,
        metallic: 1.0,
        ..Default::default()
    },
);

world.set(entity, MaterialRef::new("brass"));
```

An entity names the material the same way it names the mesh. Two entities
naming `"brass"` share one material, and changing it changes both, which is
the point.

## Changing one at runtime

Write through the registry, never through a copy:

```rust
material_registry_mutate(registry, "brass", |material| {
    material.roughness = 0.8;
});
```

The mutate path stamps the change the renderer diffs against. A material you
edited by hand and put back reaches the GPU on no frame at all.

| Field | Range | Reads as |
| --- | --- | --- |
| `base_color` | 0 to 1, four channels | The colour before light |
| `roughness` | 0 to 1 | Sharp to broad reflection |
| `metallic` | 0 to 1 | Dielectric to conductor |
| `emissive_factor` | 0 upward | Light the surface makes itself |

[Surfaces](surfaces.md) covers what those numbers do to the image, and
[Lighting](lighting.md) covers what lights them.
