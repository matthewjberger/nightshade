# nightshade-renderer

A GPU-driven wgpu renderer with a built-in frame graph. It renders one owned
input structure per frame, so any engine or tool that can fill out a
`RenderInputs` can drive it. It runs on DX12, Metal, Vulkan, and WebGPU.

This is the renderer behind the [nightshade](https://crates.io/crates/nightshade)
engine, extracted so it can be driven on its own.

```toml
[dependencies]
nightshade-renderer = "0.56.0"
```

## The seam: one input per frame

The renderer owns the scene as its own freecs ECS world. Every frame the host
composes a single [`RenderInputs`] value (that scene world plus settings, the
mesh and texture caches, and the cameras to dispatch), calls `render_frame`, and
reads back a [`FrameOutputs`] with the writes that belong to the host. A host
that runs its own freecs group composes the renderer's schema in as a member
world, so a render entity is the host entity and no scene data copies across the
boundary. A standalone host stands the world up from the registry and spawns
into it. Either way the passes and the frame graph are monomorphized on one
concrete `RenderInputs` type, and the large scene state moves in and out by
value rather than being cloned.

```text
        host (any scene representation)
                 │  compose_frame()
                 ▼
        RenderInputs  ──────────►  render_frame(renderer, &mut inputs, &mut outputs)
          scene_world (freecs)              │
          settings / scene: RendererState   │  graph executes per view
          mesh_cache / texture_cache        │  (mesh, shadows, sky, post, UI…)
          frame: cameras + commands         ▼
                 ◄──────────────────  FrameOutputs (viewport sizes, cleared flags)
        host restores the moved-in state for the next frame
```

## Getting started

Runnable examples drive the renderer directly through a small windowing harness
in `examples/common/driver.rs`, a fly camera and a frame loop built from free
functions and plain data, with no engine layer. Each example fills a
[`SingleViewHost`] of renderer data using the raw renderer and freecs API
(`world.spawn`, `RenderMaterials::add`, `mesh_cache_insert`, `world.set`).

- **`pbr`**: a grid of spheres from dielectric to metal and rough to smooth
  under a directional sun and a procedural sky.
- **`spinning`**: a field of primitives, each spun every frame by writing its
  `Transform` straight into the scene world.
- **`shadows`**: mixed shapes on a floor, tagged as shadow casters so the sun
  casts shadows across it.

Run one with `cargo run --example pbr --features wgpu,hdr`. Setting
`NS_SHOT=<path>` renders a few frames offscreen and writes a PNG instead of
opening a window (add the `screenshot` feature).

The three steps every host repeats:

1. **Bring the renderer up once** over anything with window handles.
   `WgpuRenderer::new_async(window, width, height)` builds the device, surface,
   and the built-in scene passes. The graph is left open so a host can add its
   own passes. `presentation::install_presentation_passes(&mut renderer)`
   appends the presentation tail (viewport blit, swapchain compose, UI) and
   compiles the graph. Call it once, after creation.
2. **Compose a `RenderInputs` and call `render_frame`.** Persistent state
   (`scene_world`, `scene: RendererState`, `mesh_cache`, `texture_cache`) lives
   on the host and moves in by value each frame. `frame: FrameInputs` carries the
   cameras to dispatch, drained commands, and this frame's dirty state. The
   renderer-owned fields `ibl_views` and `shadow_atlas` are composed as
   `Default::default()`, and the frame driver fills them itself.
3. **Apply `FrameOutputs` and take the persistent state back out** of the inputs
   for the next frame. On a skipped frame (occluded or lost surface) the
   unconsumed state stays in the inputs so nothing is lost.

`RenderInputs` has no `Default`: every field is required and the move-in and
move-out cycle is the contract. Write one compose function against the struct
and the compiler names any field left out. The examples' harness does this in
`examples/common/driver.rs`, and [`SingleViewHost`] with `render_single_view`
wraps the move-in and move-out cycle for a single-view host.

For the full design (the frame flow, every `RenderInputs` sub-struct, the
module map, the feature gating, and how the nightshade engine layer drives the
renderer) read [ARCHITECTURE.md](docs/ARCHITECTURE.md).

## What it renders

- Clustered forward PBR with the metallic-roughness workflow and the glTF KHR
  material extension set
- Cascaded shadow maps for the sun plus a shelf-packed atlas for spotlight and
  area-light shadows
- HDR environment maps and procedural atmospheres with prefiltered IBL,
  including day and night snapshot blending
- Skeletal animation and morph targets through a GPU skinning path
- Order-independent transparency, screen-space ambient occlusion, global
  illumination, and reflections
- Signed distance field text from a dynamic glyph atlas
- Optional grass and terrain passes behind feature flags

## GPU-driven drawing

Renderable transforms, materials, and instance data live in persistent per-world
GPU buffers updated by deltas. Frustum and hi-z occlusion culling, LOD selection,
batch table construction, and indirect draw argument population run in compute.
Draw submission is multi-draw indirect, with the draw count itself sourced from
the GPU on hardware with `MULTI_DRAW_INDIRECT_COUNT`. Bindless material textures
are used where the device supports them.

Because the draw arguments are populated on the GPU across the culling and batch
passes, a scene needs a couple of frames of warmup before its first draws land.
A host that renders continuously is unaffected, but a one-shot capture should
render a few frames before reading back.

## Using the frame graph alone

The renderer is built on its own frame graph, exposed as the `rendergraph`
module and usable without the renderer:

```toml
[dependencies]
nightshade-renderer = { version = "0.56.0", default-features = false, features = ["rendergraph"] }
```

Declare passes and the resources they read and write, compile once, execute
every frame. The graph is generic over an inputs type `C` you define. Passes
implement `PassNode<C>` and receive `&C` during prepare and execute. The graph
handles pass ordering from declared dependencies, transient resource allocation
and aliasing, load and store op optimization, dead pass culling, per-pass enable
toggles, external per-frame resources like the swapchain, and execute phases so
cached-viewport frames can run a cheap compose-only subset.

## Features

`default = ["wgpu"]`

| Feature | What it adds |
|---------|--------------|
| `wgpu` | Default. The renderer itself, on DX12, Metal, Vulkan, and WebGPU. Implies `rendergraph` and `text` |
| `rendergraph` | The frame graph alone: pass scheduling, transient resource aliasing, store-op optimization |
| `text` | The cosmic-text font engine and glyph shaping. Implied by `wgpu` |
| `hdr` | HDR decoding and image loading for skyboxes and IBL captures |
| `screenshot` | GPU readback capture to PNG (used by NS_SHOT headless capture) |
| `egui` | An egui overlay pass |
| `debug_render` | Line, bounding volume, and normal overlays plus the selection outline |
| `grass` | GPU grass pass |
| `terrain` | Terrain passes, implies `grass` |
| `browser_yield` | Cooperative yields during startup for the browser event loop |

## License

Dual-licensed under MIT or Apache-2.0, at your option.
