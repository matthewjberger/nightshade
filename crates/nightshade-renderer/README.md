# nightshade-renderer

A GPU-driven wgpu renderer with a built-in frame graph. It renders one owned input structure per frame and never sees the caller's scene representation, so any engine or tool that can fill out `RenderInputs` can drive it.

This is the renderer behind the [nightshade](https://crates.io/crates/nightshade) engine. It runs on DX12, Metal, Vulkan, and WebGPU.

```toml
[dependencies]
nightshade-renderer = "0.50.0"
```

## Design

- **One input contract.** The caller composes a `RenderInputs` value each frame: the scene snapshot, materials, mesh and texture caches, per-camera snapshots, settings, and drained render commands. The renderer consumes it and hands world-facing writes back through `FrameOutputs`. Large state moves in and out rather than being cloned.
- **GPU-driven drawing.** Object transforms, materials, and instance data live in persistent per-world GPU buffers updated by deltas. Frustum and hi-z occlusion culling, LOD selection, batch table construction, and indirect draw argument population run in compute. Draw submission is multi-draw indirect, with the draw count itself sourced from the GPU on hardware with `MULTI_DRAW_INDIRECT_COUNT`. Bindless material textures are used where the device supports them.
- **Composable passes.** The built-in pipeline is a set of `PassNode<RenderInputs>` implementations on a rendergraph: depth prepass, cascaded and atlas shadows, clustered forward PBR meshes and skinned meshes, sky and IBL, water, particles, decals, SSAO, SSGI, SSR, bloom, TAA, depth of field, post-processing, text, retained UI, and debug overlays. Callers add their own passes to the same graph.

## What it renders

- Clustered forward PBR with the metallic-roughness workflow and the glTF KHR material extension set
- Cascaded shadow maps for the sun plus a shelf-packed atlas for spotlight and area-light shadows
- HDR environment maps and procedural atmospheres with prefiltered IBL, including day and night snapshot blending
- Skeletal animation and morph targets through a GPU skinning path
- Order-independent transparency, screen-space ambient occlusion, global illumination, and reflections
- Signed distance field text from a dynamic glyph atlas
- Optional grass and terrain passes behind feature flags

## Using the frame graph alone

The renderer is built on its own frame graph, exposed as the `rendergraph` module. To use just the graph without the renderer:

```toml
[dependencies]
nightshade-renderer = { version = "0.50.0", default-features = false, features = ["rendergraph"] }
```

Declare passes and the resources they read and write, compile once, execute every frame. The graph is generic over an inputs type `C` that you define; passes implement `PassNode<C>` and receive `&C` during prepare and execute. The graph handles pass ordering from declared dependencies, transient resource allocation and aliasing, load and store op optimization, dead pass culling, per-pass enable toggles, external per-frame resources like the swapchain, and execute phases so cached-viewport frames can run a cheap compose-only subset.

```rust,ignore
use nightshade_renderer::rendergraph::{render_graph_new, render_graph_add_pass, render_graph_compile, render_graph_execute};

let mut graph = render_graph_new();

let depth = render_graph_add_depth_texture(&mut graph, "depth")
    .size(1920, 1080)
    .clear_depth(0.0)
    .transient();

let scene_color = render_graph_add_color_texture(&mut graph, "scene_color")
    .format(wgpu::TextureFormat::Rgba16Float)
    .size(1920, 1080)
    .clear_color(wgpu::Color::BLACK)
    .transient();

let swapchain = render_graph_add_color_texture(&mut graph, "swapchain")
    .format(wgpu::TextureFormat::Bgra8Unorm)
    .external();

render_graph_add_pass(&mut graph, Box::new(mesh_pass), &[
    ("color", scene_color),
    ("depth", depth),
])?;

render_graph_add_pass(&mut graph, Box::new(post_process_pass), &[
    ("input", scene_color),
    ("output", swapchain),
])?;

render_graph_compile(&mut graph)?;

// Per frame: register the swapchain texture, then execute.
render_graph_set_external_texture(&mut graph, swapchain, Some(texture), view, width, height);
let command_buffers = render_graph_execute(&mut graph, &device, &queue, &inputs)?;
queue.submit(command_buffers);
```

## Features

`default = ["wgpu"]`

| Feature | What it adds |
|---------|--------------|
| `wgpu` | Default. The renderer itself, on DX12, Metal, Vulkan, and WebGPU |
| `rendergraph` | The frame graph alone: pass scheduling, transient resource aliasing, store-op optimization |
| `assets` | HDR decoding and image loading for skyboxes and IBL captures |
| `screenshot` | GPU readback capture to PNG |
| `egui` | An egui overlay pass |
| `debug_render` | Line, bounding volume, and normal overlays plus the selection outline |
| `grass` | GPU grass pass |
| `terrain` | Terrain passes, implies `grass` |

## License

Dual-licensed under MIT or Apache-2.0, at your option.
