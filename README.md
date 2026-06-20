# template_api_leptos

A [nightshade-api](https://crates.io/crates/nightshade-api) app with a [Leptos](https://leptos.dev) frontend. The engine runs in a web worker on an `OffscreenCanvas`, off the browser's main thread, and the scene and per-frame logic are written against the `nightshade-api` facade: plain data and free functions, no user-side ECS. A Leptos UI on the main thread overlays it and talks to the worker through typed `postMessage`. Copy this directory to start a project.

This is the same architecture as `template_leptos`, with one difference: the worker drives the engine through `nightshade-api` instead of the raw `nightshade` engine. Compare the two `worker/` crates to see what the facade buys you.

## Run

```sh
just run       # native webview window over the web bundle
just run-web   # serve in the browser via trunk
```

`just init` installs the pinned toolchain (rust, wasm-bindgen, wasm-opt, trunk) through mise. Both paths render through WebGPU, so use a browser or platform webview with WebGPU and OffscreenCanvas-in-workers support (Chromium 113+, Firefox 141+).

## How it works

Two threads, one seam. The page never touches the engine and the worker never touches the DOM; everything crosses through the `protocol` messages.

- `protocol/` is the wire format: the `ClientMessage` (page to worker) and `WorkerMessage` (worker to page) enums, shared by both sides.
- `worker/` is the engine. `src/lib.rs` owns the `OffscreenCanvas`, builds the renderer, and runs the offscreen frame loop; it decodes `ClientMessage`s and injects them as engine input. `src/state.rs` holds the plain `Scene` data, and `src/systems/` is the game logic as straight-line `nightshade-api` calls (`setup`, `example`, `picking`).
- `src/` is the page. `app.rs` composes the components and forwards keyboard input, `bridge.rs` spawns the worker and turns its messages into signal writes, `state.rs` is the page state as `Copy` signals, and `components/` holds the viewport canvas, the HUD, and the loader.

The page transfers its canvas to the worker once, then forwards pointer, touch, wheel, and keyboard input. The worker streams back the adapter, FPS, entity and cube counts, and the current selection. Grow it by adding a `protocol` message, handling it in `bridge.rs` and `worker/src/lib.rs`, and building the UI under `components/`.

## nightshade-api in a worker

The facade's `run!`/`open()` entry points own a window on the main thread, which a worker cannot use. So the worker drives the engine's offscreen frame loop directly (`initialize_offscreen`, `tick_offscreen`, `resize_offscreen`) and writes the scene and systems against the facade:

- `setup.rs` calls `set_background`, `show_grid`, `orbit_camera`, `spawn_floor`, and `spawn_cube`.
- `example.rs` calls `rotate`, `set_color`, `delta_time`, and `key_pressed`.
- `picking.rs` calls `entity_under_cursor`, the facade's synchronous ray pick.

The `Scene` is plain data implementing the engine's `State` trait, the one piece of plumbing the offscreen loop needs. Only the parts the facade does not expose from a worker drop to the raw engine: the renderer, the offscreen driver, input injection, and the selection outline.

## Desktop

`desktop/` is a native shell: it serves the built `dist/` on an ephemeral localhost port and opens it in a `wry` webview window. `just run` builds the bundle and launches it. Debug builds read `../dist` from disk; release builds embed it into the executable (`just build-desktop`).

## License

Dual-licensed under MIT or Apache-2.0, at your option.
