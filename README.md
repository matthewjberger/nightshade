# nightshade-template-leptos

A template for building [Nightshade](https://github.com/matthewjberger/nightshade) apps with the Leptos/webview architecture. The whole engine runs inside a web worker against an OffscreenCanvas and renders through WebGPU off the main thread. A [Leptos](https://leptos.dev) UI drives it from the main thread, and a native webview shell turns the same bundle into a desktop app.

This is the same architecture as the [nightshade-viewer](https://github.com/matthewjberger/nightshade-viewer) and the Nightshade editor, reduced to a skeleton you can grow a game from. For a plain winit/native template instead, see the engine's `template/` crate.

## Workspace

- `protocol`, the message and data types both sides share, plus the postMessage envelope keys.
- `worker`, the wasm module inside the web worker. The engine `World` plus a `TemplateWorld` (its own `freecs` world) driven by system functions in `worker/src/systems/`.
- the root crate (`page`), the Leptos UI. A viewport that transfers the canvas and forwards input, an example HUD, and a two-directional bridge over grouped signal state.
- `desktop`, the native shell: a webview window over the web bundle, served from an ephemeral localhost port.

## Quickstart

Tooling is pinned in [mise.toml](mise.toml). Install [mise](https://mise.jdx.dev) and [just](https://github.com/casey/just), then:

```bash
just init
just run
```

`just run` builds the worker, builds the bundle with Trunk, and opens the app in a native webview window. `just run-web` serves the same bundle at http://127.0.0.1:8080 instead. The browser path needs WebGPU and OffscreenCanvas-in-workers support (Chromium 113+, Firefox 141+). The worker compiles the whole engine, so the first build is large.

## How it fits together

The page and the worker share nothing but messages, defined once in `protocol/src/lib.rs`. The page sends `ClientMessage` (forwarded pointer, touch, wheel, and keyboard input, plus your game messages) and the worker answers with `WorkerMessage` (renderer facts, stats, plus your game messages). Every message rides in a `{ message }` envelope so transferables like the canvas can travel next to it.

The worker side mirrors the native template: a `Template` state shell in `worker/src/state.rs` forwards each engine `State` hook to free functions in `worker/src/systems/`, and `worker/src/ecs.rs` declares the user-side `freecs` world for game components and resources. The example system spins every spawned cube and spawns another on Space or when the HUD button sends `SpawnCube`.

Picking is wired end to end as a working example of the round trip: a click or tap without drag sends `Pick`, the worker requests a GPU pick (`gpu_picking.request_pick`), polls the readback a frame later in `worker/src/systems/picking.rs`, drives the engine's selection outline, and answers with `Selected` for the HUD. Clicking the background clears the selection.

To add a feature, work the seam end to end:

1. Add a variant to `ClientMessage` or `WorkerMessage` in `protocol/src/lib.rs`.
2. Handle it in `worker/src/lib.rs::apply_client_message` (page to worker) or post it with `post` (worker to page).
3. Handle it in `src/bridge.rs` and build the UI in a new file under `src/components/`.

For binary payloads (dropped files, save data), large lists, gizmos, and the MCP agent surface, lift the patterns from the viewer and the editor. They are the same architecture with those features built out.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
