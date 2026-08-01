# nightshade-graphics

This crate is the CPU half of drawing. It holds the resources the render description lives in, the registry and cache commands a program calls to make a material or a mesh, and the frame systems that sync the scene, the transforms, the materials, the lights, the skins and the bounding volumes into the state a backend reads. It names no GPU API, so a backend composes it and reads what it produced.

## Composing it

```rust
App::new()
    .add_plugin(nightshade_platform::PlatformPlugin)
    .add_plugin(nightshade_scene::ScenePlugin)
    .add_plugin(nightshade_graphics::GraphicsPlugin)
    .run()
```

`GraphicsPlugin` inserts the render resources, among them `RenderSettings`, `MaterialRegistry`, `MeshCache`, `TextureCache`, `Viewport`, `ScheduledCapture` and `CommandQueues`, and registers the frame systems across `Stage::FrameStart`, `Stage::FrameUpdate`, `Stage::FramePostUpdate`, `Stage::Render` and `Stage::Last`. `WindowPlugin` and both backends already compose it, so a program composing `DefaultPlugins` has it.

## Using it

```rust
use nightshade_graphics::commands::builtins::spawn_mesh_at;
use nightshade_graphics::commands::material_registry::insert_material;
use nightshade_graphics::prelude::*;

fn spawn_floor(registry: &mut MaterialRegistry, commands: &mut Commands) {
    let entity = spawn_mesh_at(commands, "Plane", Vec3::zeros(), Vec3::new(20.0, 1.0, 20.0));
    let material = insert_material(registry, "floor", textures::tiled_material(FLOOR_TEXTURE, 8.0));
    commands.set(entity, MaterialRef::new(material));
}
```

`spawn_mesh_at` and its siblings `spawn_cube_at`, `spawn_sphere_at`, `spawn_plane_at` and `spawn_sun` stand a primitive up at a place with a scale. `insert_material` registers a material under a name and reports the handle a `MaterialRef` carries, and a material already registered is changed through `material_registry_mutate` so the edit reaches the backend. `queue_render_command` and `queue_ecs_command` push onto the two queues in `CommandQueues`, which is how a texture upload, a skybox load or a screenshot is asked for. The crate's prelude re-exports the `nightshade-render` prelude beside these, so a system file names one import for both.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-graphics = { path = "../nightshade-graphics", default-features = false, features = ["contract"] }
```

The contract carries the commands, the queries and the resources without the plugin and the frame systems, which is how `nightshade-gltf` names `TextureRecipe` and `TextureProductionPlan` while linking none of the behavior.
