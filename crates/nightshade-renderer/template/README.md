# nightshade-renderer standalone starter

A single-file starter that drives [`nightshade-renderer`](../README.md)
directly. The window, fly camera, scene, and render loop all live in `main.rs`
with no engine layer and no shared harness — copy that one file plus the
dependencies in `Cargo.toml` into a new binary crate and it runs.

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

The frame loop keeps its persistent state in a `SingleViewHost` and draws with
`render_single_view`, which rebuilds the whole scene every frame. As a scene
grows, give each mesh a `RenderBounds` component to tighten shadow and frustum
culling, and drive the renderer's incremental delta path instead of the full
rebuild.

## Using it outside this repo

The `Cargo.toml` here depends on the renderer by path for local development.
Copy the directory elsewhere and point the dependency at a published version:

```toml
nightshade-renderer = { version = "0.56", features = ["wgpu", "hdr"] }
```
