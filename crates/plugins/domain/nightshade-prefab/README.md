# nightshade-prefab

`nightshade-prefab` says what a prefab is and turns one into rows. A `Prefab` is a tree of nodes carrying a transform and the components a node wants, among them a mesh, a material, a light and a camera, and spawning it builds the whole hierarchy with the parent links already in place. It knows no file format, so a crate that reads a model hands it the `ModelImport` it produced and this crate does the spawning.

## Composing it

The crate carries no plugin. It installs no resource and registers no system, so a program calls its functions where it wants a prefab on the screen.

The registries the spawn writes into belong to `nightshade-graphics` and `nightshade-render`, which a program already has when it opens a window.

## Using it

```rust
let (middle, reach) = nightshade_prefab::queries::prefab_reach(&prefab, &import.meshes);
let mut spawn = nightshade_prefab::PrefabSpawn { storage, resources };
nightshade_prefab::queue_model_load(&mut spawn, &mut import);
nightshade_prefab::spawn_prefab(&mut spawn, &prefab, Vec3::zeros());
```

`PrefabSpawn` is the pair of borrows every call takes, the storage the rows land in and the resource map the meshes, materials and textures register against. `queue_model_load` hands the import's geometry, materials and images to those registries, and `spawn_prefab` then builds the tree at an offset and returns the root entity. `spawn_prefab_with_animations` and `spawn_prefab_with_skins` do the same for a model that carries clips or a skeleton, and `spawn_prefab_with_source` stamps a `PrefabSource` on the root so the group can be found again. `prefab_bounds` and `prefab_reach` measure a prefab before it is spawned, which is how an app frames a camera on a model it has not placed yet, and `find_group_root` walks up from any entity to the prefab it belongs to.

## Taking it from another crate

The crate does not split. Its node types name the render schema, so it cannot compile without it and the whole crate is taken.

```toml
nightshade-prefab = { path = "../nightshade-prefab" }
```
