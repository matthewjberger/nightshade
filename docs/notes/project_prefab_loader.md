---
name: project_prefab_loader
description: A model is read through nightshade-gltf's Loading resource on a worker crew, and nightshade-prefab turns what lands into rows
type: project
---

Two crates share the one way an app puts a model in the world, and each owns a
different half.

`nightshade-gltf` carries `GltfPlugin` and the read. `Loading` is
`nightshade_load::Loader<Gltf>`, the plugin inserts it, and
`collect_the_models` is the one system that merges what the crew hands back. An
app calls `load` with a `Source`, which is `Path`, `Bytes`, `Parts` or, behind
the `fetch` feature, `Url`, and gets a `LoadHandle` at once. `ready` reports
without taking and `take` hands the parse over once, which is how an app waits
for two models before placing either. The crew is hired inside
`Loader::default`, so the read and the parse never run on the frame.

`nightshade-prefab` carries `PrefabPlugin` and the spawn. It knows no file
format: it takes the `ModelImport` the read produced and writes rows through
`spawn_prefab` and its four longer forms, which carry animations, a source, or
skins. `PrefabSpawn` is the pair of borrows those take, the storage and the
resources together. `PrefabPlugin` composes `ScenePlugin`, because a spawned
prefab is a tree and something has to propagate it.

`nightshade-load` itself is at `crates/support/`. It names no engine: it is the
handle, the crew and the web fetch, and `Loader<R>` is generic over whatever a
caller can read. The capability that names the reader installs it, so
`GltfPlugin` inserts `Loader<Gltf>` and `AudioPlugin` inserts `Loader<Sound>`.

An app keeps the handle in its own resource and does its tail, framing the
camera, gathering collision, parenting a rig, on the frame the model lands.
