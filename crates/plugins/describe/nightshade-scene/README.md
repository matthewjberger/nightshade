# nightshade-scene

The scene crate holds what a node is and where it is. `Name`, `Visibility`, `Guid`, `LocalTransform`, `GlobalTransform`, `Rotation`, `IgnoreParentScale` and `Lifetime` are declared here, and the crate composes world matrices over the parent-child tree that `nightshade-ecs` stores as `ChildOf` and `HierarchyIndex`. A capability that wants a transform or a name takes this crate rather than declaring its own.

## Composing it

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(MyGamePlugin)
    .run()
```

`WindowPlugin` already composes `ScenePlugin`, so a program that takes `DefaultPlugins` has it. The plugin inserts `EntityRegistry` and `TransformState`, then registers `update_global_transforms_system` and `expire_lifetimes_system` on `Stage::FramePostUpdate`.

## Using it

```rust
use nightshade_scene::prelude::*;

let local_transform = LocalTransform {
    translation,
    ..Default::default()
};
commands.spawn((
    Name(name.to_string()),
    local_transform,
    GlobalTransform(local_transform.as_matrix()),
    shape,
))
```

A row that carries a `LocalTransform` and a `GlobalTransform` is a node, and the propagation pass rewrites the global matrix from the local one and the parent's each frame. Write the local transform through a stamping path, with `world.set`, `world.get_mut::<LocalTransform>` or a `&mut` query element, so the change reaches the pass and the renderer. `update_parent` reparents a node and `propagate_subtree` walks one branch on demand. `queries` answers the questions about the tree: `find_entity_by_name`, `query_children`, `query_descendants`, `query_ancestors`, `is_ancestor_of`, `topmost`, `local_transform_under`, `parent_world_matrix` and `chain_from_root_to_leaf`. A `Lifetime` counts down and the expiry system despawns the entity when it reaches zero.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-scene = { path = "../nightshade-scene", default-features = false, features = ["contract"] }
```

The contract carries the node vocabulary and nothing that runs: the transforms, the rotation, the lifetime, the primitives `Name`, `Visibility` and `Guid`, and the re-exported `ChildOf`. It pulls in no `nightshade`, so naming a transform never links the propagation pass.
