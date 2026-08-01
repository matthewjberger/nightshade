# nightshade-gltf

This crate reads glTF into the `ModelImport` that `nightshade-prefab` spawns. It owns the import itself, the meshes, materials, textures, skins, animation clips, cameras and lights, and it owns the reading of the bytes, on a worker crew from a path or from memory and over the network behind the `fetch` feature. A program that wants a model asks the `Loading` resource for it and either takes the result back itself or lets the plugin spawn it.

## Composing it

```rust
app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
    .add_plugin(nightshade_decals::DecalsPlugin)
    .add_plugin(nightshade_gltf::GltfPlugin)
    .run()
```

`GltfPlugin` inserts the `Loading` resource, which is a `nightshade_load::Loader<Gltf>`, and registers `systems::loading::collect_the_models` on `Stage::Update`. That system gathers what landed, holds the scheduled capture back while a read is in flight, and spawns every prefab of an import whose note carries a position.

## Using it

```rust
use nightshade_gltf::queries::sample_model;

pub fn initialize(mut resources: nightshade::ecs::system_param::Resources) {
    let handle = nightshade_load::load(
        resources.res_mut::<nightshade_gltf::Loading>(),
        nightshade_load::Source::Path(sample_model(theme::MODEL)),
    );
    resources.res_mut::<DebugViewsResources>().model = Some(handle);
}
```

`load` hands back a `LoadHandle` right away and the read runs on the crew. A later system calls `nightshade_load::take` with that handle to get the `Result<ModelImport, String>` and stands the scene up itself, which is what an app does when it wants to place the camera around what arrived. An app that only wants the model in the world calls `load_noted` with the position as the note and lets `collect_the_models` do the spawn. `queries::sample_model`, `sample_model_parts` and `sample_model_url` name a model in the Khronos sample set by path or by URL.

## Taking it from another crate

The crate carries no `contract` feature, because nothing asks it for types. A program that reads a model names it whole, and one that only spawns a prefab it already holds names `nightshade-prefab` instead.
