# nightshade-renderer standalone starter

A single-file starter that drives [`nightshade-renderer`](../README.md)
directly. The window, fly camera, scene, and render loop all live in
`standalone.rs` with no engine layer and no shared harness — copy that one file
plus the dependencies in `Cargo.toml` into a new binary crate and it runs.

## Run

```sh
just run
```

or with cargo directly:

```sh
cargo run --release
```

## Controls

W/A/S/D move, Q/E down/up, mouse to look, Esc quits.

## What it shows

A procedural sky and a directional sun over a floor of colored shapes. Each
shape spins in place (a per-frame `world.set` of its `Transform`) and casts a
shadow (a `ShadowCaster` tag). The scene is built entirely with the raw renderer
and freecs API: `world.spawn` of component tuples, `RenderMaterials::add`, and
`mesh_cache_insert`.

## Using it outside this repo

The `Cargo.toml` here depends on the renderer by path for local development.
Copy the directory elsewhere and point the dependency at a published version:

```toml
nightshade-renderer = { version = "0.56", features = ["wgpu", "hdr"] }
```
