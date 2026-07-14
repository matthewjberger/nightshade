# nightshade-renderer

A GPU-driven wgpu renderer with a built-in frame graph. It renders one owned input structure per frame and never sees the caller's scene representation, so any engine or tool that can fill out `RenderInputs` can drive it.

This is the renderer behind the [nightshade](https://crates.io/crates/nightshade) engine. It runs on DX12, Metal, Vulkan, and WebGPU.

```toml
[dependencies]
nightshade-renderer = "0.53.0"
```

## Design

- **One input contract.** The caller composes a `RenderInputs` value each frame: the scene snapshot, materials, mesh and texture caches, per-camera snapshots, settings, and drained render commands. The renderer consumes it and hands world-facing writes back through `FrameOutputs`. Large state moves in and out rather than being cloned.
- **GPU-driven drawing.** Object transforms, materials, and instance data live in persistent per-world GPU buffers updated by deltas. Frustum and hi-z occlusion culling, LOD selection, batch table construction, and indirect draw argument population run in compute. Draw submission is multi-draw indirect, with the draw count itself sourced from the GPU on hardware with `MULTI_DRAW_INDIRECT_COUNT`. Bindless material textures are used where the device supports them.
- **Composable passes.** The built-in pipeline is a set of `PassNode<RenderInputs>` implementations on a rendergraph: depth prepass, cascaded and atlas shadows, clustered forward PBR meshes and skinned meshes, sky and IBL, water, particles, decals, SSAO, SSGI, SSR, bloom, TAA, depth of field, post-processing, text, retained UI, and debug overlays. Callers add their own passes to the same graph.

## Driving it

The default frame driver is `wgpu::frame::render_frame(renderer, &mut inputs, &mut outputs)`. It owns the whole per-frame sequence: render command drain, IBL capture, dirty-state handoff to the passes, surface acquisition with occlusion and loss handling, per-camera dispatch with viewport caching, UI composite, and present.

A host drives it with three steps per frame:

1. Compose a `RenderInputs`. Persistent state (`scene: RendererState`, the mesh and texture caches, materials) lives on your side and moves in by value each frame; keep it current with deltas rather than rebuilding it. Frame data (`frame: FrameInputs`) carries the cameras to dispatch, drained commands, dirty spheres, and debug lines for this frame.
2. Call `render_frame`. On a skipped frame (occluded or lost surface) the unconsumed dirty state stays in the inputs.
3. Apply `FrameOutputs` (viewport texture sizes, cleared force-render flags, `frame_executed`) and take the persistent state back out of the inputs for the next frame.

`RenderInputs` deliberately has no `Default`: every field is load-bearing and the move-in/move-out cycle is the contract. Build one compose function against the struct and the compiler walks you through the fields.

In code, the whole cycle:

```rust,ignore
use nightshade_renderer::config::{DebugDraw, RenderSettings, RendererState};
use nightshade_renderer::core::create_renderer;
use nightshade_renderer::mesh_cache::MeshCache;
use nightshade_renderer::wgpu::frame::render_frame;
use nightshade_renderer::wgpu::render_configs::{FrameInputs, FrameOutputs, RenderInputs};
use nightshade_renderer::wgpu::texture_cache::TextureCache;

// Once: bring the renderer up over anything with window handles
// (a winit window on native, an OffscreenCanvas on the web).
let mut renderer = create_renderer(window, width, height).await?;

// Persistent state lives on your side between frames. Keep it current
// with deltas: insert meshes into the cache, queue texture uploads,
// update the scene snapshot's transforms and materials.
let mut scene = RendererState::default();
let mut mesh_cache = MeshCache::default();
let mut texture_cache = TextureCache::default();

loop {
    // 1. Compose this frame's inputs. Persistent state moves in by value;
    //    renderer-owned fields (ibl_views, shadow_atlas) compose as
    //    Default::default(). The frame field carries this frame's cameras
    //    (as CameraFrameInputs), drained commands, and dirty spheres.
    let mut inputs = RenderInputs {
        settings: RenderSettings::default(),
        debug_draw: DebugDraw::default(),
        scene: std::mem::take(&mut scene),
        ibl_views: Default::default(),
        shadow_atlas: Default::default(),
        wind: Default::default(),
        mesh_cache: std::mem::take(&mut mesh_cache),
        texture_cache: std::mem::take(&mut texture_cache),
        pending_particle_textures: Vec::new(),
        ui: Default::default(),
        view: view_inputs,
        world_id: 0,
        frame: compose_frame_inputs(cameras, commands),
    };

    // 2. Render. On a skipped frame (occluded or lost surface) the
    //    unconsumed dirty state stays in the inputs.
    let mut outputs = FrameOutputs::default();
    render_frame(&mut renderer, &mut inputs, &mut outputs)?;

    // 3. Apply the outputs and take the persistent state back out.
    apply_viewport_sizes(&outputs.viewport_texture_sizes);
    scene = inputs.scene;
    mesh_cache = inputs.mesh_cache;
    texture_cache = inputs.texture_cache;
}
```

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
nightshade-renderer = { version = "0.53.0", default-features = false, features = ["rendergraph"] }
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
| `hdr` | HDR decoding and image loading for skyboxes and IBL captures |
| `screenshot` | GPU readback capture to PNG |
| `egui` | An egui overlay pass |
| `debug_render` | Line, bounding volume, and normal overlays plus the selection outline |
| `grass` | GPU grass pass |
| `terrain` | Terrain passes, implies `grass` |

## License

Dual-licensed under MIT or Apache-2.0, at your option.
